namespace LLMAssistant.Game.Dialogue;

public class DialogueNode
{
    public string Id { get; set; } = "";
    public string? SpeakerId { get; set; }
    public string Text { get; set; } = "";
    public string? NextNodeId { get; set; }
    public List<DialogueChoice> Choices { get; set; } = [];
    public string? Condition { get; set; }
    public string? Script { get; set; }
    public string? Emotion { get; set; }
    public Dictionary<string, string> Metadata { get; set; } = new();

    // LLM integration
    public bool UseLLM { get; set; }
    public string? LLMPrompt { get; set; }
    public string? LLMSystemPrompt { get; set; }
}

public class DialogueChoice
{
    public string Text { get; set; } = "";
    public string? NextNodeId { get; set; }
    public string? Condition { get; set; }
    public string? Script { get; set; }
    public float? RelationshipDelta { get; set; }
    public Dictionary<string, float> RelationshipChanges { get; set; } = new();
}
