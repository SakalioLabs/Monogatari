namespace LLMAssistant.AI.API;

public class APIConfig
{
    public string BaseUrl { get; set; } = "https://api.openai.com/v1";
    public string ApiKey { get; set; } = "";
    public string Model { get; set; } = "gpt-3.5-turbo";
    public int MaxTokens { get; set; } = 512;
    public float Temperature { get; set; } = 0.7f;
    public float TopP { get; set; } = 0.9f;
    public int TimeoutSeconds { get; set; } = 60;
    public Dictionary<string, string> Headers { get; set; } = new();
}
