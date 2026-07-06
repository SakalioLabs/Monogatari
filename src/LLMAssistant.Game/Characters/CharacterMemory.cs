namespace LLMAssistant.Game.Characters;

public class CharacterMemory
{
    private readonly List<MemoryEntry> _memories = [];
    private readonly int _maxMemories;

    public IReadOnlyList<MemoryEntry> Memories => _memories;

    public CharacterMemory(int maxMemories = 100)
    {
        _maxMemories = maxMemories;
    }

    public void AddMemory(string content, MemoryType type, float importance = 0.5f)
    {
        _memories.Add(new MemoryEntry
        {
            Content = content,
            Type = type,
            Importance = importance,
            Timestamp = DateTime.UtcNow,
            AccessCount = 0
        });

        if (_memories.Count > _maxMemories)
        {
            var toRemove = _memories
                .OrderBy(m => m.Importance * (1 + m.AccessCount * 0.1))
                .First();
            _memories.Remove(toRemove);
        }
    }

    public List<MemoryEntry> Recall(string query, int maxResults = 5)
    {
        var keywords = query.Split(' ', StringSplitOptions.RemoveEmptyEntries);

        return _memories
            .Select(m => new
            {
                Memory = m,
                Score = keywords.Count(k =>
                    m.Content.Contains(k, StringComparison.OrdinalIgnoreCase)) * m.Importance
            })
            .Where(x => x.Score > 0)
            .OrderByDescending(x => x.Score)
            .Take(maxResults)
            .Select(x =>
            {
                x.Memory.AccessCount++;
                x.Memory.LastAccessed = DateTime.UtcNow;
                return x.Memory;
            })
            .ToList();
    }

    public List<MemoryEntry> GetRecent(int count = 10)
    {
        return _memories
            .OrderByDescending(m => m.Timestamp)
            .Take(count)
            .ToList();
    }

    public void ForgetOld(TimeSpan maxAge)
    {
        _memories.RemoveAll(m => DateTime.UtcNow - m.Timestamp > maxAge);
    }
}

public class MemoryEntry
{
    public string Content { get; set; } = "";
    public MemoryType Type { get; set; }
    public float Importance { get; set; }
    public DateTime Timestamp { get; set; }
    public DateTime? LastAccessed { get; set; }
    public int AccessCount { get; set; }
}

public enum MemoryType
{
    Conversation,
    Event,
    Emotion,
    Knowledge,
    Relationship
}
