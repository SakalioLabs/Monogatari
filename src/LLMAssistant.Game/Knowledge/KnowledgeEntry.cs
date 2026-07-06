namespace LLMAssistant.Game.Knowledge;

public class KnowledgeEntry
{
    public string Id { get; set; } = "";
    public string Category { get; set; } = "";
    public string Title { get; set; } = "";
    public string Content { get; set; } = "";
    public List<string> Tags { get; set; } = [];
    public float Importance { get; set; } = 0.5f;
    public Dictionary<string, string> Metadata { get; set; } = new();
    public List<string> RelatedEntries { get; set; } = [];
}

public class KnowledgeCategory
{
    public string Name { get; set; } = "";
    public string Description { get; set; } = "";
    public List<KnowledgeEntry> Entries { get; set; } = [];
}
