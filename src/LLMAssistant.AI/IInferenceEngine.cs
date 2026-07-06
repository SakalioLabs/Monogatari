namespace LLMAssistant.AI;

public interface IInferenceEngine
{
    string Name { get; }
    bool IsReady { get; }
    Task<bool> InitializeAsync();
    Task<InferenceResult> InferAsync(string prompt, InferenceOptions? options = null);
    IAsyncEnumerable<string> InferStreamAsync(string prompt, InferenceOptions? options = null);
    void Shutdown();
}

public class InferenceOptions
{
    public int MaxTokens { get; set; } = 512;
    public float Temperature { get; set; } = 0.7f;
    public float TopP { get; set; } = 0.9f;
    public int TopK { get; set; } = 50;
    public float RepetitionPenalty { get; set; } = 1.1f;
    public string[]? StopSequences { get; set; }
}
