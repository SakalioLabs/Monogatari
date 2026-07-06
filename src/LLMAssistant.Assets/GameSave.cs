namespace LLMAssistant.Assets;

public class GameSave
{
    public string SaveId { get; set; } = Guid.NewGuid().ToString();
    public string SaveName { get; set; } = "";
    public DateTime Timestamp { get; set; } = DateTime.UtcNow;
    public string CurrentScene { get; set; } = "";
    public string CurrentDialogueId { get; set; } = "";
    public string CurrentNodeId { get; set; } = "";
    public Dictionary<string, string> Variables { get; set; } = new();
    public Dictionary<string, CharacterSaveData> Characters { get; set; } = new();
    public List<string> Flags { get; set; } = [];
}

public class CharacterSaveData
{
    public string CurrentEmotion { get; set; } = "neutral";
    public Dictionary<string, float> Relationships { get; set; } = new();
    public List<string> MemoryContents { get; set; } = [];
}
