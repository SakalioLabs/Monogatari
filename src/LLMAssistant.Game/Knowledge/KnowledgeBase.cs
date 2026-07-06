using System.Text.Json;
using LLMAssistant.Core;
using LLMAssistant.Core.Services;

namespace LLMAssistant.Game.Knowledge;

public class KnowledgeBase : IGameService
{
    private readonly Dictionary<string, KnowledgeEntry> _entries = new();
    private readonly Dictionary<string, KnowledgeCategory> _categories = new();
    private readonly Dictionary<string, List<string>> _tagIndex = new();

    public string ServiceName => "KnowledgeBase";
    public IReadOnlyDictionary<string, KnowledgeEntry> Entries => _entries;
    public IReadOnlyDictionary<string, KnowledgeCategory> Categories => _categories;

    public void AddEntry(KnowledgeEntry entry)
    {
        _entries[entry.Id] = entry;

        if (!_categories.TryGetValue(entry.Category, out var category))
        {
            category = new KnowledgeCategory { Name = entry.Category };
            _categories[entry.Category] = category;
        }
        category.Entries.Add(entry);

        foreach (var tag in entry.Tags)
        {
            if (!_tagIndex.TryGetValue(tag, out var tagEntries))
            {
                tagEntries = [];
                _tagIndex[tag] = tagEntries;
            }
            if (!tagEntries.Contains(entry.Id))
                tagEntries.Add(entry.Id);
        }
    }

    public KnowledgeEntry? GetEntry(string id)
    {
        return _entries.TryGetValue(id, out var entry) ? entry : null;
    }

    public List<KnowledgeEntry> Search(string query, int maxResults = 10)
    {
        var keywords = query.Split(' ', StringSplitOptions.RemoveEmptyEntries);

        return _entries.Values
            .Select(e => new
            {
                Entry = e,
                Score = CalculateRelevance(e, keywords)
            })
            .Where(x => x.Score > 0)
            .OrderByDescending(x => x.Score)
            .Take(maxResults)
            .Select(x => x.Entry)
            .ToList();
    }

    public List<KnowledgeEntry> GetByCategory(string category)
    {
        return _categories.TryGetValue(category, out var cat)
            ? cat.Entries
            : [];
    }

    public List<KnowledgeEntry> GetByTag(string tag)
    {
        if (!_tagIndex.TryGetValue(tag, out var ids))
            return [];

        return ids.Select(id => GetEntry(id))
            .Where(e => e != null)
            .Cast<KnowledgeEntry>()
            .ToList();
    }

    public List<KnowledgeEntry> GetRelated(string entryId, int maxResults = 5)
    {
        var entry = GetEntry(entryId);
        if (entry == null) return [];

        return entry.RelatedEntries
            .Select(id => GetEntry(id))
            .Where(e => e != null)
            .Cast<KnowledgeEntry>()
            .Take(maxResults)
            .ToList();
    }

    private float CalculateRelevance(KnowledgeEntry entry, string[] keywords)
    {
        float score = 0;

        foreach (var keyword in keywords)
        {
            if (entry.Title.Contains(keyword, StringComparison.OrdinalIgnoreCase))
                score += 3.0f;
            if (entry.Content.Contains(keyword, StringComparison.OrdinalIgnoreCase))
                score += 1.0f;
            if (entry.Tags.Any(t => t.Contains(keyword, StringComparison.OrdinalIgnoreCase)))
                score += 2.0f;
        }

        return score * entry.Importance;
    }

    public async Task LoadFromDirectory(string directoryPath)
    {
        if (!Directory.Exists(directoryPath)) return;

        foreach (var file in Directory.GetFiles(directoryPath, "*.json"))
        {
            try
            {
                var json = await File.ReadAllTextAsync(file);
                var entries = JsonSerializer.Deserialize<List<KnowledgeEntry>>(json, JsonOptions.Default);
                if (entries != null)
                {
                    foreach (var entry in entries)
                    {
                        AddEntry(entry);
                    }
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to load knowledge from {file}: {ex.Message}");
            }
        }
    }

    public void Initialize() { }
    public void Update(double deltaTime) { }
    public void Shutdown() { }
}
