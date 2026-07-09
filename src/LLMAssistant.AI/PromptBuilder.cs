using System.Text;
using LLMAssistant.Game.Characters;
using LLMAssistant.Game.Knowledge;

namespace LLMAssistant.AI;

public class PromptBuilder
{
    private static readonly string[] PromptControlRoles = ["system", "developer", "user", "assistant", "tool"];
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
            prompt.AppendLine(SanitizePromptContent(_systemPrompt.ToString()));
            prompt.AppendLine();
        }
        foreach (var (role, content) in _messages)
        {
            var header = SafeRoleHeader(role);
            prompt.AppendLine($"[{header}]");
            prompt.AppendLine(SanitizePromptContent(content));
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

    private static string SafeRoleHeader(string role)
    {
        var normalized = role.Trim().ToLowerInvariant();
        return normalized switch
        {
            "assistant" => "Assistant",
            "system" => "System",
            _ => "User"
        };
    }

    private static string SanitizePromptContent(string content)
    {
        return string.Join(
            Environment.NewLine,
            content.Replace("\r\n", "\n").Replace('\r', '\n').Split('\n').Select(SanitizePromptLine));
    }

    private static string SanitizePromptLine(string line)
    {
        var trimmed = line.Trim();
        var asciiLower = trimmed.ToLowerInvariant();
        var normalizedLower = NormalizeSecurityText(trimmed);

        if (IsBracketRoleMarker(asciiLower))
        {
            return line.Replace('[', '{').Replace(']', '}');
        }
        if (IsBracketRoleMarker(normalizedLower))
        {
            return normalizedLower.Replace('[', '{').Replace(']', '}');
        }
        if (IsRoleCodeFenceLine(asciiLower)
            || IsRoleCodeFenceLine(normalizedLower)
            || IsStructuralRoleControlLine(TrimPromptLinePrefixes(asciiLower))
            || IsStructuralRoleControlLine(TrimPromptLinePrefixes(normalizedLower)))
        {
            return "Guarded prompt-control marker omitted.";
        }

        return line.Replace("\0", "");
    }

    private static string NormalizeSecurityText(string content)
    {
        var normalized = new StringBuilder(content.Length);
        var previousWasWhitespace = false;

        foreach (var ch in content)
        {
            var mapped = NormalizeSecurityChar(ch);
            if (mapped is null)
            {
                continue;
            }

            foreach (var lowered in char.ToLowerInvariant(mapped.Value).ToString())
            {
                if (char.IsWhiteSpace(lowered))
                {
                    if (!previousWasWhitespace)
                    {
                        normalized.Append(' ');
                        previousWasWhitespace = true;
                    }
                }
                else
                {
                    normalized.Append(lowered);
                    previousWasWhitespace = false;
                }
            }
        }

        return normalized.ToString();
    }

    private static char? NormalizeSecurityChar(char ch)
    {
        return ch switch
        {
            '\u00AD' or '\u034F' or '\u061C' or '\u180E' => null,
            >= '\u200B' and <= '\u200F' => null,
            >= '\u202A' and <= '\u202E' => null,
            >= '\u2060' and <= '\u206F' => null,
            '\uFEFF' => null,
            '\u3000' => ' ',
            >= '\uFF01' and <= '\uFF5E' => (char)(ch - 0xFEE0),
            _ => ch
        };
    }

    private static bool IsBracketRoleMarker(string line)
    {
        return PromptControlRoles.Any(role =>
        {
            var marker = $"[{role}]";
            return line == marker || (line.StartsWith(marker, StringComparison.Ordinal) && char.IsWhiteSpace(line[marker.Length]));
        });
    }

    private static string TrimPromptLinePrefixes(string line)
    {
        var trimmed = line.TrimStart();
        if (trimmed.StartsWith("<!--", StringComparison.Ordinal))
        {
            trimmed = trimmed[4..];
        }

        return trimmed.TrimStart(' ', '\t', '>', '!', '/', '-', '*', '`', '#', '"', '\'');
    }

    private static bool IsStructuralRoleControlLine(string line)
    {
        var compact = new string(line.Where(ch => !char.IsWhiteSpace(ch)).ToArray());

        foreach (var role in PromptControlRoles)
        {
            if (ContainsRoleTag(line, compact, role)
                || compact.Contains($"\"role\":\"{role}\"", StringComparison.Ordinal)
                || compact.Contains($"'role':'{role}'", StringComparison.Ordinal)
                || compact.Contains($"role=\"{role}\"", StringComparison.Ordinal)
                || compact.Contains($"role='{role}'", StringComparison.Ordinal))
            {
                return true;
            }

            if (RoleHeadingMatches(line, role))
            {
                return true;
            }
        }

        return false;
    }

    private static bool RoleHeadingMatches(string line, string role)
    {
        if (!line.StartsWith(role, StringComparison.Ordinal))
        {
            return false;
        }

        var rest = line[role.Length..].TrimStart();
        if (rest.Length == 0 || RoleHeadingSeparator(rest))
        {
            return true;
        }

        foreach (var label in new[] { "message", "messages", "instruction", "instructions", "prompt", "prompts" })
        {
            if (!rest.StartsWith(label, StringComparison.Ordinal))
            {
                continue;
            }

            var afterLabel = rest[label.Length..].TrimStart();
            if (afterLabel.Length == 0 || RoleHeadingSeparator(afterLabel))
            {
                return true;
            }
        }

        return false;
    }

    private static bool RoleHeadingSeparator(string value)
    {
        return value.StartsWith(':') || value.StartsWith('=') || value.StartsWith("=>", StringComparison.Ordinal);
    }

    private static bool IsRoleCodeFenceLine(string line)
    {
        var payload = RoleCodeFencePayload(line, '`') ?? RoleCodeFencePayload(line, '~');
        return payload is not null && PromptControlRoles.Any(role => RoleLabelWithBoundary(payload, role));
    }

    private static string? RoleCodeFencePayload(string line, char fence)
    {
        var trimmed = line.TrimStart();
        var markerLength = 0;
        while (markerLength < trimmed.Length && trimmed[markerLength] == fence)
        {
            markerLength++;
        }

        if (markerLength < 3)
        {
            return null;
        }

        return trimmed[markerLength..].TrimStart();
    }

    private static bool RoleLabelWithBoundary(string line, string role)
    {
        if (!line.StartsWith(role, StringComparison.Ordinal))
        {
            return false;
        }

        if (line.Length == role.Length)
        {
            return true;
        }

        return !char.IsAsciiLetterOrDigit(line[role.Length]);
    }

    private static bool ContainsRoleTag(string line, string compact, string role)
    {
        return compact.Contains($"<{role}>", StringComparison.Ordinal)
            || compact.Contains($"</{role}>", StringComparison.Ordinal)
            || compact.Contains($"<{role}/", StringComparison.Ordinal)
            || compact.Contains($"<{role}:", StringComparison.Ordinal)
            || ContainsRoleTagWithBoundary(line, $"<{role}")
            || ContainsRoleTagWithBoundary(line, $"</{role}");
    }

    private static bool ContainsRoleTagWithBoundary(string line, string marker)
    {
        var searchFrom = 0;
        while (searchFrom < line.Length)
        {
            var offset = line.IndexOf(marker, searchFrom, StringComparison.Ordinal);
            if (offset < 0)
            {
                return false;
            }

            var boundary = offset + marker.Length;
            if (boundary >= line.Length)
            {
                return true;
            }

            var ch = line[boundary];
            if (char.IsWhiteSpace(ch) || ch is '>' or '/' or ':')
            {
                return true;
            }

            searchFrom = boundary;
        }

        return false;
    }
}
