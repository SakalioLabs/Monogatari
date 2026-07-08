using System.Net.Http.Headers;
using System.Runtime.CompilerServices;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;

namespace LLMAssistant.AI.API;

public class APIEngine : IInferenceEngine
{
    private HttpClient? _httpClient;
    private readonly APIConfig _config;
    private static readonly Regex TokenLikeValueRegex = new(
        @"\b(?:sk-[A-Za-z0-9_-]{20,}|ghp_[A-Za-z0-9_]{20,}|github_pat_[A-Za-z0-9_]{20,})\b",
        RegexOptions.Compiled);
    private static readonly Regex BearerTokenRegex = new(
        @"(?i)\b(Bearer\s+)([A-Za-z0-9._~+/=-]{8,})",
        RegexOptions.Compiled);
    private static readonly Regex SecretJsonAssignmentRegex = new(
        @"(?i)([""']?(?:api[_-]?key|apikey|access[_-]?token|accesstoken|token|secret|password|authorization)[""']?\s*:\s*[""'])([^""']*)([""'])",
        RegexOptions.Compiled);
    private static readonly Regex SecretQueryAssignmentRegex = new(
        @"(?i)\b((?:api[_-]?key|apikey|access[_-]?token|accesstoken|token|secret|password|authorization)=)([^&\s,;}\]]+)",
        RegexOptions.Compiled);
    private static readonly Regex SecretHeaderAssignmentRegex = new(
        @"(?i)\b((?:api[_-]?key|apikey|access[_-]?token|accesstoken|token|secret|password|authorization)\s*:\s*)([^\r\n,;]+)",
        RegexOptions.Compiled);

    public string Name => $"API ({_config.Model})";
    public bool IsReady => _httpClient != null;

    public APIEngine(APIConfig config)
    {
        _config = config;
    }

    public Task<bool> InitializeAsync()
    {
        try
        {
            _httpClient = new HttpClient
            {
                BaseAddress = new Uri(_config.BaseUrl),
                Timeout = TimeSpan.FromSeconds(_config.TimeoutSeconds)
            };

            if (!string.IsNullOrEmpty(_config.ApiKey))
            {
                _httpClient.DefaultRequestHeaders.Authorization =
                    new AuthenticationHeaderValue("Bearer", _config.ApiKey);
            }

            foreach (var header in _config.Headers)
            {
                _httpClient.DefaultRequestHeaders.TryAddWithoutValidation(header.Key, header.Value);
            }

            Console.WriteLine($"API Engine initialized: {RedactSensitiveText(_config.BaseUrl)}");
            return Task.FromResult(true);
        }
        catch (Exception ex)
        {
            Console.WriteLine($"API initialization failed: {RedactSensitiveText(ex.Message)}");
            return Task.FromResult(false);
        }
    }

    public async Task<InferenceResult> InferAsync(string prompt, InferenceOptions? options = null)
    {
        if (_httpClient == null)
            return InferenceResult.Fail("API client not initialized");

        options ??= new InferenceOptions();

        try
        {
            var sw = System.Diagnostics.Stopwatch.StartNew();

            var requestBody = new
            {
                model = _config.Model,
                messages = new[]
                {
                    new { role = "user", content = prompt }
                },
                max_tokens = options.MaxTokens,
                temperature = options.Temperature,
                top_p = options.TopP
            };

            var json = JsonSerializer.Serialize(requestBody);
            var content = new StringContent(json, Encoding.UTF8, "application/json");

            var response = await _httpClient.PostAsync("/chat/completions", content);
            var responseBody = await response.Content.ReadAsStringAsync();

            sw.Stop();

            if (!response.IsSuccessStatusCode)
            {
                return InferenceResult.Fail($"API error ({response.StatusCode}): {RedactSensitiveText(responseBody)}");
            }

            var result = JsonSerializer.Deserialize<JsonElement>(responseBody);
            var text = result
                .GetProperty("choices")[0]
                .GetProperty("message")
                .GetProperty("content")
                .GetString() ?? "";

            var tokensUsed = result.TryGetProperty("usage", out var usage)
                ? usage.GetProperty("completion_tokens").GetInt32()
                : 0;

            return InferenceResult.Ok(text.Trim(), sw.Elapsed.TotalMilliseconds, tokensUsed);
        }
        catch (Exception ex)
        {
            return InferenceResult.Fail($"API request failed: {RedactSensitiveText(ex.Message)}");
        }
    }

    public static string RedactSensitiveText(string text)
    {
        if (string.IsNullOrEmpty(text))
        {
            return text;
        }

        var redacted = SecretJsonAssignmentRegex.Replace(text, "$1<redacted>$3");
        redacted = SecretQueryAssignmentRegex.Replace(redacted, "$1<redacted>");
        redacted = SecretHeaderAssignmentRegex.Replace(redacted, "$1<redacted>");
        redacted = BearerTokenRegex.Replace(redacted, "$1<redacted>");
        redacted = TokenLikeValueRegex.Replace(redacted, "<redacted>");
        return redacted;
    }

    public async IAsyncEnumerable<string> InferStreamAsync(string prompt, InferenceOptions? options = null)
    {
        if (_httpClient == null)
        {
            yield break;
        }

        options ??= new InferenceOptions();

        var requestBody = new
        {
            model = _config.Model,
            messages = new[]
            {
                new { role = "user", content = prompt }
            },
            max_tokens = options.MaxTokens,
            temperature = options.Temperature,
            top_p = options.TopP,
            stream = true
        };

        var json = JsonSerializer.Serialize(requestBody);
        var content = new StringContent(json, Encoding.UTF8, "application/json");

        var request = new HttpRequestMessage(HttpMethod.Post, "/chat/completions")
        {
            Content = content
        };

        var response = await _httpClient.SendAsync(request, HttpCompletionOption.ResponseHeadersRead);
        var stream = await response.Content.ReadAsStreamAsync();
        using var reader = new StreamReader(stream);

        while (!reader.EndOfStream)
        {
            var line = await reader.ReadLineAsync();
            if (string.IsNullOrEmpty(line) || !line.StartsWith("data: ")) continue;

            var data = line["data: ".Length..];
            if (data == "[DONE]") break;

            var chunk = JsonSerializer.Deserialize<JsonElement>(data);
            var delta = chunk.GetProperty("choices")[0].GetProperty("delta");

            if (delta.TryGetProperty("content", out var contentElement))
            {
                var text = contentElement.GetString();
                if (!string.IsNullOrEmpty(text))
                {
                    yield return text;
                }
            }
        }
    }

    public void Shutdown()
    {
        _httpClient?.Dispose();
        _httpClient = null;
    }
}
