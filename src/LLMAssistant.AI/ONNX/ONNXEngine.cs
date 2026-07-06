using Microsoft.ML.OnnxRuntime;
using Microsoft.ML.OnnxRuntime.Tensors;
using LLMAssistant.Core.Events;

namespace LLMAssistant.AI.ONNX;

public class ONNXEngine : IInferenceEngine
{
    private InferenceSession? _session;
    private readonly ModelConfig _config;
    private readonly Tokenizer _tokenizer;

    public string Name => "ONNX DirectML";
    public bool IsReady => _session != null;

    public ONNXEngine(ModelConfig config)
    {
        _config = config;
        _tokenizer = new Tokenizer();
    }

    public Task<bool> InitializeAsync()
    {
        try
        {
            if (!File.Exists(_config.ModelPath))
            {
                Console.WriteLine($"Model file not found: {_config.ModelPath}");
                return Task.FromResult(false);
            }

            var options = new SessionOptions();
            options.LogSeverityLevel = OrtLoggingLevel.ORT_LOGGING_LEVEL_WARNING;

            if (_config.UseDirectML)
            {
                try
                {
                    options.AppendExecutionProvider_DML(_config.DeviceId);
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"DirectML not available, falling back to CPU: {ex.Message}");
                }
            }
            options.AppendExecutionProvider_CPU();

            _session = new InferenceSession(_config.ModelPath, options);

            if (!string.IsNullOrEmpty(_config.TokenizerPath))
            {
                _tokenizer.LoadFromJson(_config.TokenizerPath);
            }

            Console.WriteLine($"ONNX Engine initialized: {_config.ModelPath}");
            return Task.FromResult(true);
        }
        catch (Exception ex)
        {
            Console.WriteLine($"ONNX initialization failed: {ex.Message}");
            return Task.FromResult(false);
        }
    }

    public Task<InferenceResult> InferAsync(string prompt, InferenceOptions? options = null)
    {
        if (_session == null)
            return Task.FromResult(InferenceResult.Fail("ONNX session not initialized"));

        options ??= new InferenceOptions();

        try
        {
            var sw = System.Diagnostics.Stopwatch.StartNew();
            var inputTokens = _tokenizer.Encode(prompt);
            var generatedTokens = new List<int>(inputTokens);

            var inputIds = new DenseTensor<long>(
                inputTokens.Select(t => (long)t).ToArray(),
                [1, inputTokens.Length]);

            var attentionMask = new DenseTensor<long>(
                Enumerable.Repeat(1L, inputTokens.Length).ToArray(),
                [1, inputTokens.Length]);

            for (int i = 0; i < options.MaxTokens; i++)
            {
                var inputs = new List<NamedOnnxValue>
                {
                    NamedOnnxValue.CreateFromTensor("input_ids", inputIds),
                    NamedOnnxValue.CreateFromTensor("attention_mask", attentionMask)
                };

                var results = _session.Run(inputs);
                var outputTensor = results.First().AsTensor<float>();
                var vocabSize = outputTensor.Dimensions[^1];
                var lastLogits = new float[vocabSize];

                var offset = (inputIds.Dimensions[1] - 1) * vocabSize;
                for (int v = 0; v < vocabSize; v++)
                {
                    lastLogits[v] = outputTensor.GetValue(offset + v);
                }

                var nextToken = SampleToken(lastLogits, options.Temperature, options.TopP, options.TopK);
                generatedTokens.Add(nextToken);

                // Check for EOS
                if (nextToken == 2 || (options.StopSequences != null &&
                    options.StopSequences.Any(s => _tokenizer.DecodeToken(nextToken).Contains(s))))
                {
                    break;
                }

                // Prepare next input
                inputIds = new DenseTensor<long>(
                    generatedTokens.Select(t => (long)t).ToArray(),
                    [1, generatedTokens.Count]);

                attentionMask = new DenseTensor<long>(
                    Enumerable.Repeat(1L, generatedTokens.Count).ToArray(),
                    [1, generatedTokens.Count]);
            }

            sw.Stop();
            var outputTokens = generatedTokens.Skip(inputTokens.Length).ToArray();
            var outputText = _tokenizer.Decode(outputTokens);

            return Task.FromResult(InferenceResult.Ok(outputText, sw.Elapsed.TotalMilliseconds, outputTokens.Length));
        }
        catch (Exception ex)
        {
            return Task.FromResult(InferenceResult.Fail($"Inference error: {ex.Message}"));
        }
    }

    public async IAsyncEnumerable<string> InferStreamAsync(string prompt, InferenceOptions? options = null)
    {
        var result = await InferAsync(prompt, options);
        if (result.Success)
        {
            var words = result.Text.Split(' ');
            foreach (var word in words)
            {
                yield return word + " ";
                await Task.Delay(50);
            }
        }
    }

    private int SampleToken(float[] logits, float temperature, float topP, int topK)
    {
        // Apply temperature
        if (Math.Abs(temperature - 1.0f) > 0.001f)
        {
            for (int i = 0; i < logits.Length; i++)
                logits[i] /= temperature;
        }

        // Top-K filtering
        var indexed = logits.Select((v, i) => (v, i)).OrderByDescending(x => x.v).Take(topK).ToList();

        // Top-P (nucleus) filtering
        var total = 0.0f;
        var probs = new List<(float prob, int idx)>();
        foreach (var (v, i) in indexed)
        {
            var prob = MathF.Exp(v);
            probs.Add((prob, i));
            total += prob;
        }

        var cumSum = 0.0f;
        var filtered = new List<(float prob, int idx)>();
        foreach (var (prob, idx) in probs.OrderByDescending(p => p.prob))
        {
            cumSum += prob / total;
            filtered.Add((prob, idx));
            if (cumSum >= topP) break;
        }

        // Sample
        var rand = Random.Shared.NextSingle();
        var sum = 0.0f;
        var filteredTotal = filtered.Sum(f => f.prob);
        foreach (var (prob, idx) in filtered)
        {
            sum += prob / filteredTotal;
            if (rand <= sum) return idx;
        }

        return filtered[0].idx;
    }

    public void Shutdown()
    {
        _session?.Dispose();
        _session = null;
    }
}
