using System.Text;
using LLMAssistant.Game.Characters;

namespace LLMAssistant.Game.Knowledge;

public class ContextBuilder
{
    private readonly KnowledgeBase _knowledgeBase;
    private readonly CharacterManager _characterManager;

    public ContextBuilder(KnowledgeBase knowledgeBase, CharacterManager characterManager)
    {
        _knowledgeBase = knowledgeBase;
        _characterManager = characterManager;
    }

    public string BuildContext(string characterId, string playerInput, int maxKnowledgeItems = 5)
    {
        var context = new StringBuilder();
        var character = _characterManager.GetCharacter(characterId);

        if (character != null)
        {
            context.AppendLine("=== CHARACTER CONTEXT ===");
            context.AppendLine(character.BuildSystemPrompt());
            context.AppendLine();
        }

        var relevantKnowledge = _knowledgeBase.Search(playerInput, maxKnowledgeItems);
        if (relevantKnowledge.Count > 0)
        {
            context.AppendLine("=== RELEVANT KNOWLEDGE ===");
            foreach (var entry in relevantKnowledge)
            {
                context.AppendLine($"[{entry.Category}] {entry.Title}: {entry.Content}");
            }
            context.AppendLine();
        }

        if (character != null)
        {
            var memories = character.Memory.Recall(playerInput, 3);
            if (memories.Count > 0)
            {
                context.AppendLine("=== CHARACTER MEMORIES ===");
                foreach (var memory in memories)
                {
                    context.AppendLine($"- {memory.Content}");
                }
                context.AppendLine();
            }
        }

        return context.ToString();
    }

    public string BuildFullWorldContext()
    {
        var context = new StringBuilder();

        context.AppendLine("=== WORLD KNOWLEDGE ===");
        foreach (var category in _knowledgeBase.Categories.Values)
        {
            context.AppendLine($"\n## {category.Name}");
            foreach (var entry in category.Entries)
            {
                context.AppendLine($"- {entry.Title}: {entry.Content}");
            }
        }

        return context.ToString();
    }
}
