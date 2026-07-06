namespace LLMAssistant.Game.Dialogue;

public class DialogueScript
{
    public string Id { get; set; } = "";
    public string Title { get; set; } = "";
    public string Description { get; set; } = "";
    public string StartNodeId { get; set; } = "";
    public List<DialogueNode> Nodes { get; set; } = [];
    public Dictionary<string, string> Variables { get; set; } = new();

    public DialogueNode? GetNode(string nodeId)
    {
        return Nodes.FirstOrDefault(n => n.Id == nodeId);
    }
}
