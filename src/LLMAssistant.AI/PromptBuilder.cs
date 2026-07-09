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
        var sanitized = new List<string>();
        PromptControlBlock? activeBlock = null;

        foreach (var line in content.Replace("\r\n", "\n").Replace('\r', '\n').Split('\n'))
        {
            if (activeBlock is { } block)
            {
                if (PromptControlBlockEnds(line, block))
                {
                    activeBlock = null;
                }
                continue;
            }

            var blockStart = PromptControlBlockStartForLine(line);
            if (blockStart is { } start)
            {
                sanitized.Add("Guarded prompt-control marker omitted.");
                if (!start.ClosesOnLine)
                {
                    activeBlock = start.Block;
                }
                continue;
            }

            sanitized.Add(SanitizePromptLine(line));
        }

        return string.Join(Environment.NewLine, sanitized);
    }

    private enum PromptControlBlockKind
    {
        Fence,
        RoleTag,
        HtmlComment,
        CComment
    }

    private readonly struct PromptControlBlock(PromptControlBlockKind kind, char fence = '\0', string? role = null)
    {
        public PromptControlBlockKind Kind { get; } = kind;
        public char Fence { get; } = fence;
        public string? Role { get; } = role;
    }

    private readonly struct PromptControlBlockStart(PromptControlBlock block, bool closesOnLine)
    {
        public PromptControlBlock Block { get; } = block;
        public bool ClosesOnLine { get; } = closesOnLine;
    }

    private static PromptControlBlockStart? PromptControlBlockStartForLine(string line)
    {
        var trimmed = line.Trim();
        var asciiLower = trimmed.ToLowerInvariant();
        var normalizedLower = NormalizeSecurityText(trimmed);

        return PromptControlBlockStartForNormalizedLine(asciiLower)
            ?? PromptControlBlockStartForNormalizedLine(normalizedLower);
    }

    private static PromptControlBlockStart? PromptControlBlockStartForNormalizedLine(string line)
    {
        if (RoleCodeFenceChar(line) is { } fence)
        {
            return new PromptControlBlockStart(
                new PromptControlBlock(PromptControlBlockKind.Fence, fence),
                closesOnLine: false);
        }

        var trimmed = line.TrimStart();
        var controlLine = TrimPromptLinePrefixes(trimmed);
        if (trimmed.StartsWith("<!--", StringComparison.Ordinal)
            && IsStructuralRoleControlLine(controlLine))
        {
            return new PromptControlBlockStart(
                new PromptControlBlock(PromptControlBlockKind.HtmlComment),
                trimmed.Contains("-->", StringComparison.Ordinal));
        }
        if (trimmed.StartsWith("/*", StringComparison.Ordinal)
            && IsStructuralRoleControlLine(controlLine))
        {
            return new PromptControlBlockStart(
                new PromptControlBlock(PromptControlBlockKind.CComment),
                trimmed.Contains("*/", StringComparison.Ordinal));
        }

        var compact = new string(controlLine.Where(ch => !char.IsWhiteSpace(ch)).ToArray());
        foreach (var role in PromptControlRoles)
        {
            if (!ContainsRoleOpeningTag(controlLine, compact, role))
            {
                continue;
            }

            return new PromptControlBlockStart(
                new PromptControlBlock(PromptControlBlockKind.RoleTag, role: role),
                ContainsRoleClosingTag(controlLine, compact, role)
                    || compact.Contains("/>", StringComparison.Ordinal));
        }

        return null;
    }

    private static bool PromptControlBlockEnds(string line, PromptControlBlock block)
    {
        var trimmed = line.Trim();
        var asciiLower = trimmed.ToLowerInvariant();
        var normalizedLower = NormalizeSecurityText(trimmed);

        return PromptControlBlockEndsInNormalizedLine(asciiLower, block)
            || PromptControlBlockEndsInNormalizedLine(normalizedLower, block);
    }

    private static bool PromptControlBlockEndsInNormalizedLine(string line, PromptControlBlock block)
    {
        return block.Kind switch
        {
            PromptControlBlockKind.Fence => IsCodeFenceBoundary(line, block.Fence),
            PromptControlBlockKind.RoleTag => block.Role is { } role
                && ContainsRoleClosingTag(line, new string(line.Where(ch => !char.IsWhiteSpace(ch)).ToArray()), role),
            PromptControlBlockKind.HtmlComment => line.Contains("-->", StringComparison.Ordinal),
            PromptControlBlockKind.CComment => line.Contains("*/", StringComparison.Ordinal),
            _ => false
        };
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
        return RoleCodeFenceChar(line) is not null;
    }

    private static char? RoleCodeFenceChar(string line)
    {
        foreach (var fence in new[] { '`', '~' })
        {
            var payload = RoleCodeFencePayload(line, fence);
            if (payload is not null && PromptControlRoles.Any(role => RoleLabelWithBoundary(payload, role)))
            {
                return fence;
            }
        }

        return null;
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

    private static bool IsCodeFenceBoundary(string line, char fence)
    {
        var trimmed = line.TrimStart();
        var markerLength = 0;
        while (markerLength < trimmed.Length && trimmed[markerLength] == fence)
        {
            markerLength++;
        }

        return markerLength >= 3;
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

    private static bool ContainsRoleOpeningTag(string line, string compact, string role)
    {
        return compact.Contains($"<{role}>", StringComparison.Ordinal)
            || compact.Contains($"<{role}/", StringComparison.Ordinal)
            || compact.Contains($"<{role}:", StringComparison.Ordinal)
            || ContainsRoleTagWithBoundary(line, $"<{role}");
    }

    private static bool ContainsRoleClosingTag(string line, string compact, string role)
    {
        return compact.Contains($"</{role}>", StringComparison.Ordinal)
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
