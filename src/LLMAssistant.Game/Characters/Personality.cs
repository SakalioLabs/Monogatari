namespace LLMAssistant.Game.Characters;

public class Personality
{
    public string Name { get; set; } = "";
    public string Description { get; set; } = "";
    public Dictionary<string, float> Traits { get; set; } = new();
    public List<string> Likes { get; set; } = [];
    public List<string> Dislikes { get; set; } = [];
    public string SpeechStyle { get; set; } = "neutral";
    public Dictionary<string, string> EmotionStates { get; set; } = new();

    public float GetTrait(string traitName)
    {
        return Traits.TryGetValue(traitName, out var value) ? value : 0.5f;
    }

    public void SetTrait(string traitName, float value)
    {
        Traits[traitName] = Math.Clamp(value, 0f, 1f);
    }

    public string GetEmotionState(string emotion)
    {
        return EmotionStates.TryGetValue(emotion, out var state) ? state : "neutral";
    }

    public Personality Clone()
    {
        return new Personality
        {
            Name = Name,
            Description = Description,
            Traits = new Dictionary<string, float>(Traits),
            Likes = [.. Likes],
            Dislikes = [.. Dislikes],
            SpeechStyle = SpeechStyle,
            EmotionStates = new Dictionary<string, string>(EmotionStates)
        };
    }
}
