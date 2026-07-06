using LLMAssistant.Game.Knowledge;

namespace LLMAssistant.Tests;

public class KnowledgeBaseTests
{
    [Fact]
    public void AddEntry_And_Search_FindsRelevant()
    {
        var kb = new KnowledgeBase();
        kb.AddEntry(new KnowledgeEntry
        {
            Id = "test1",
            Category = "Places",
            Title = "Central Park",
            Content = "A beautiful park with cherry blossoms",
            Tags = ["park", "nature"],
            Importance = 0.8f
        });

        var results = kb.Search("cherry blossoms");
        Assert.Single(results);
        Assert.Equal("test1", results[0].Id);
    }

    [Fact]
    public void GetByCategory_ReturnsCorrectEntries()
    {
        var kb = new KnowledgeBase();
        kb.AddEntry(new KnowledgeEntry { Id = "1", Category = "Places", Title = "Park" });
        kb.AddEntry(new KnowledgeEntry { Id = "2", Category = "Places", Title = "Library" });
        kb.AddEntry(new KnowledgeEntry { Id = "3", Category = "Items", Title = "Book" });

        var places = kb.GetByCategory("Places");
        Assert.Equal(2, places.Count);
    }

    [Fact]
    public void GetByTag_FindsTaggedEntries()
    {
        var kb = new KnowledgeBase();
        kb.AddEntry(new KnowledgeEntry
        {
            Id = "1",
            Tags = ["nature", "park"]
        });
        kb.AddEntry(new KnowledgeEntry
        {
            Id = "2",
            Tags = ["city", "building"]
        });

        var nature = kb.GetByTag("nature");
        Assert.Single(nature);
        Assert.Equal("1", nature[0].Id);
    }
}
