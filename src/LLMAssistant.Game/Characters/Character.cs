using System.Text.Json.Serialization;

namespace LLMAssistant.Game.Characters;

public class Character
{
    public string Id { get; set; } = "";
    public string Name { get; set; } = "";
    public string DisplayName { get; set; } = "";
    public string Description { get; set; } = "";
    public string Background { get; set; } = "";

    [JsonIgnore]
    public Personality Personality { get; set; } = new();

    [JsonIgnore]
    public CharacterMemory Memory { get; set; } = new();

    public string CurrentEmotion { get; set; } = "neutral";
    public Dictionary<string, float> Relationship { get; set; } = new();
    public string SpritePath { get; set; } = "";
    public Dictionary<string, string> Sprites { get; set; } = new();

    public string? PersonalityJson { get; set; }

    public void UpdateEmotion(string emotion, float intensity = 1.0f)
    {
        CurrentEmotion = emotion;
        Memory.AddMemory($"Emotion changed to {emotion} (intensity: {intensity:F1})",
            MemoryType.Emotion, intensity * 0.3f);
    }

    public void AdjustRelationship(string target, float delta)
    {
        if (!Relationship.ContainsKey(target))
            Relationship[target] = 0.5f;

        Relationship[target] = Math.Clamp(Relationship[target] + delta, 0f, 1f);
    }

    public float GetRelationship(string target)
    {
        return Relationship.TryGetValue(target, out var value) ? value : 0.5f;
    }

    public string GetCurrentSprite()
    {
        if (Sprites.TryGetValue(CurrentEmotion, out var sprite))
            return sprite;
        return Sprites.TryGetValue("neutral", out var neutral) ? neutral : SpritePath;
    }

    public string BuildSystemPrompt()
    {
        var prompt = $"You are {DisplayName}. {Description}\n";
        prompt += $"Background: {Background}\n";
        prompt += $"Personality: {Personality.Description}\n";
        prompt += $"Speech style: {Personality.SpeechStyle}\n";
        prompt += $"Current emotion: {CurrentEmotion}\n";

        if (Personality.Likes.Count > 0)
            prompt += $"Likes: {string.Join(", ", Personality.Likes)}\n";
        if (Personality.Dislikes.Count > 0)
            prompt += $"Dislikes: {string.Join(", ", Personality.Dislikes)}\n";

        var recentMemories = Memory.GetRecent(5);
        if (recentMemories.Count > 0)
        {
            prompt += "Recent memories:\n";
            foreach (var mem in recentMemories)
            {
                prompt += $"- {mem.Content}\n";
            }
        }

        return prompt;
    }
}
