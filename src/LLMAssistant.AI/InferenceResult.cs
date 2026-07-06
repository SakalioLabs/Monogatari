namespace LLMAssistant.AI;

public class InferenceResult
{
    public string Text { get; set; } = "";
    public bool Success { get; set; }
    public string? Error { get; set; }
    public double DurationMs { get; set; }
    public int TokensGenerated { get; set; }
    public Dictionary<string, object> Metadata { get; set; } = new();

    public static InferenceResult Ok(string text, double durationMs = 0, int tokens = 0)
    {
        return new InferenceResult
        {
            Text = text,
            Success = true,
            DurationMs = durationMs,
            TokensGenerated = tokens
        };
    }

    public static InferenceResult Fail(string error)
    {
        return new InferenceResult
        {
            Success = false,
            Error = error
        };
    }
}
