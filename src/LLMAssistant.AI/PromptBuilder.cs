using System.Text;
using LLMAssistant.Game.Characters;
using LLMAssistant.Game.Knowledge;

namespace LLMAssistant.AI;

public class PromptBuilder
{
    private readonly StringBuilder _systemPrompt = new();
    private readonly List<(string role, string content)> _messages = new();

    public PromptBuilder WithSystemPrompt(string prompt)
    {
        _systemPrompt.AppendLine(prompt);
        return this;
    }

    public PromptBuilder WithCharacterContext(Character character)
    {
        _systemPrompt.AppendLine(character.BuildSystemPrompt());
        return this;
    }

    public PromptBuilder WithKnowledgeContext(KnowledgeBase kb, string query, int maxItems = 5)
    {
        var results = kb.Search(query, maxItems);
        foreach (var entry in results)
        {
            _systemPrompt.AppendLine($"[{entry.Category}] {entry.Title}: {entry.Content}");
        }
        return this;
    }

    public PromptBuilder WithWorldContext(ContextBuilder contextBuilder)
    {
        _systemPrompt.AppendLine(contextBuilder.BuildFullWorldContext());
        return this;
    }

    public PromptBuilder AddMessage(string role, string content)
    {
        _messages.Add((role, content));
        return this;
    }

    public PromptBuilder AddUserMessage(string content)
    {
        return AddMessage("user", content);
    }

    public PromptBuilder AddAssistantMessage(string content)
    {
        return AddMessage("assistant", content);
    }

    public string Build()
    {
        var prompt = new StringBuilder();
        if (_systemPrompt.Length > 0)
        {
            prompt.AppendLine("[System]");
            prompt.AppendLine(_systemPrompt.ToString());
            prompt.AppendLine();
        }
        foreach (var (role, content) in _messages)
        {
            var header = char.ToUpper(role[0]) + role[1..];
            prompt.AppendLine($"[{header}]");
            prompt.AppendLine(content);
            prompt.AppendLine();
        }
        prompt.AppendLine("[Assistant]");
        return prompt.ToString();
    }

    public void Clear()
    {
        _systemPrompt.Clear();
        _messages.Clear();
    }
}
