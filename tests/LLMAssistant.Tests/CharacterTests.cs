using LLMAssistant.Game.Characters;

namespace LLMAssistant.Tests;

public class CharacterTests
{
    [Fact]
    public void Personality_Clone_CreatesDeepCopy()
    {
        var original = new Personality
        {
            Name = "Test",
            Traits = new Dictionary<string, float> { ["kindness"] = 0.8f },
            Likes = ["cats"]
        };

        var clone = original.Clone();
        clone.Traits["kindness"] = 0.5f;
        clone.Likes.Add("dogs");

        Assert.Equal(0.8f, original.Traits["kindness"]);
        Assert.Single(original.Likes);
    }

    [Fact]
    public void CharacterMemory_AddAndRecall()
    {
        var memory = new CharacterMemory();
        memory.AddMemory("I love cherry blossoms", MemoryType.Knowledge, 0.8f);
        memory.AddMemory("The weather is nice today", MemoryType.Conversation, 0.5f);

        var results = memory.Recall("cherry blossoms");
        Assert.Single(results);
        Assert.Contains("cherry blossoms", results[0].Content);
    }

    [Fact]
    public void CharacterMemory_MaxMemories_EvictsLeastImportant()
    {
        var memory = new CharacterMemory(maxMemories: 3);
        memory.AddMemory("Low importance", MemoryType.Conversation, 0.1f);
        memory.AddMemory("Medium importance", MemoryType.Conversation, 0.5f);
        memory.AddMemory("High importance", MemoryType.Knowledge, 0.9f);
        memory.AddMemory("New memory", MemoryType.Event, 0.5f);

        Assert.Equal(3, memory.Memories.Count);
        Assert.DoesNotContain(memory.Memories, m => m.Content == "Low importance");
    }

    [Fact]
    public void Character_AdjustRelationship_Clamps()
    {
        var character = new Character { Id = "test" };
        character.AdjustRelationship("player", 1.5f);
        Assert.Equal(1.0f, character.GetRelationship("player"));

        character.AdjustRelationship("player", -3.0f);
        Assert.Equal(0.0f, character.GetRelationship("player"));
    }

    [Fact]
    public void Character_BuildSystemPrompt_ContainsAllInfo()
    {
        var character = new Character
        {
            DisplayName = "TestChar",
            Description = "A test character",
            Background = "Test background",
            CurrentEmotion = "happy"
        };
        character.Personality = new Personality
        {
            Description = "Friendly",
            SpeechStyle = "Casual",
            Likes = ["music"]
        };

        var prompt = character.BuildSystemPrompt();
        Assert.Contains("TestChar", prompt);
        Assert.Contains("A test character", prompt);
        Assert.Contains("happy", prompt);
        Assert.Contains("music", prompt);
    }
}
