using System.Text.Json;

namespace LLMAssistant.Core;

public static class JsonOptions
{
    public static readonly JsonSerializerOptions Default = new()
    {
        PropertyNameCaseInsensitive = true,
        WriteIndented = true
    };
}
