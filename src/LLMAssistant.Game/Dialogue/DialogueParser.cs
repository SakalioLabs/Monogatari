using System.Text.Json;
using LLMAssistant.Core;

namespace LLMAssistant.Game.Dialogue;

public static class DialogueParser
{
    public static DialogueScript? ParseFromJson(string json)
    {
        return JsonSerializer.Deserialize<DialogueScript>(json, JsonOptions.Default);
    }

    public static async Task<DialogueScript?> LoadFromFile(string filePath)
    {
        if (!File.Exists(filePath)) return null;
        var json = await File.ReadAllTextAsync(filePath);
        return ParseFromJson(json);
    }

    public static string ToJson(DialogueScript script)
    {
        return JsonSerializer.Serialize(script, new JsonSerializerOptions
        {
            WriteIndented = true
        });
    }
}
