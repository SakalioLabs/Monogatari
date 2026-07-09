using System.Text.Json;
using LLMAssistant.Game.Characters;
using LLMAssistant.Game.Knowledge;
using LLMAssistant.Game.Dialogue;
using LLMAssistant.AI;
using LLMAssistant.AI.Pipeline;
using LLMAssistant.Scripting;

namespace LLMAssistant.Tests;

public class IntegrationTests
{
    [Fact]
    public async Task LoadCharacters_FromExampleJson_Success()
    {
        var manager = new CharacterManager();
        var dataPath = Path.Combine(FindProjectRoot(), "data", "characters");
        await manager.LoadCharactersFromDirectory(dataPath);

        Assert.True(manager.Characters.Count >= 3);
        var sakura = manager.GetCharacter("sakura");
        Assert.NotNull(sakura);
        Assert.Equal("Sakura", sakura.DisplayName);
        Assert.Equal("happy", sakura.CurrentEmotion);
        Assert.Equal("assets/characters/sakura_sprite.svg", sakura.SpritePath);
        Assert.Equal(0.9f, sakura.Personality.GetTrait("openness"));
        Assert.Contains("nature metaphors", sakura.Personality.SpeechStyle, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public async Task LoadKnowledge_FromExampleJson_Success()
    {
        var kb = new KnowledgeBase();
        var dataPath = Path.Combine(FindProjectRoot(), "data", "knowledge");
        await kb.LoadFromDirectory(dataPath);

        Assert.True(kb.Entries.Count >= 3);
        Assert.True(kb.Categories.Count >= 3);

        var park = kb.GetEntry("location_park");
        Assert.NotNull(park);
        Assert.Contains("cherry blossom", park.Content, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public async Task LoadDialogue_FromExampleJson_Success()
    {
        var dataPath = Path.Combine(FindProjectRoot(), "data", "dialogue", "example_dialogue.json");
        var script = await DialogueParser.LoadFromFile(dataPath);

        Assert.NotNull(script);
        Assert.Equal("meeting_sakura", script.Id);
        Assert.Equal("Meeting Sakura", script.Title);
        Assert.NotEmpty(script.Nodes);

        var startNode = script.GetNode("start");
        Assert.NotNull(startNode);
        Assert.Equal("sakura", startNode.SpeakerId);
        Assert.Equal("intro_1", startNode.NextNodeId);
    }

    [Fact]
    public async Task DialogueManager_StartAndAdvance_FiresEvents()
    {
        var charManager = new CharacterManager();
        var dataPath = Path.Combine(FindProjectRoot(), "data", "characters");
        await charManager.LoadCharactersFromDirectory(dataPath);

        var kb = new KnowledgeBase();
        var dm = new DialogueManager(charManager, kb);

        string? shownName = null;
        string? shownText = null;
        dm.OnShowDialogue += (name, text) => { shownName = name; shownText = text; };

        var dialoguePath = Path.Combine(FindProjectRoot(), "data", "dialogue", "example_dialogue.json");
        var script = await DialogueParser.LoadFromFile(dialoguePath);
        Assert.NotNull(script);

        dm.StartDialogue(script);

        Assert.NotNull(shownName);
        Assert.NotNull(shownText);
        Assert.Equal("Sakura", shownName);
        Assert.Contains("Hello", shownText, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public async Task DialogueManager_SelectChoice_UpdatesRelationship()
    {
        var charManager = new CharacterManager();
        var dataPath = Path.Combine(FindProjectRoot(), "data", "characters");
        await charManager.LoadCharactersFromDirectory(dataPath);

        var kb = new KnowledgeBase();
        var dm = new DialogueManager(charManager, kb);

        var dialoguePath = Path.Combine(FindProjectRoot(), "data", "dialogue", "example_dialogue.json");
        var script = await DialogueParser.LoadFromFile(dialoguePath);
        Assert.NotNull(script);

        dm.StartDialogue(script);
        dm.Advance(); // move to intro_1 (has choices)

        Assert.NotNull(dm.CurrentNode);
        Assert.True(dm.CurrentNode.Choices.Count > 0);

        var sakura = charManager.GetCharacter("sakura");
        Assert.NotNull(sakura);
        var initialRel = sakura.GetRelationship("player");

        dm.SelectChoice(0); // scenery response (+0.2)

        Assert.True(sakura.GetRelationship("player") > initialRel);
    }

    [Fact]
    public void DialogueParser_StillAcceptsLegacyNodeArrays()
    {
        const string json = """
            {
              "id": "legacy",
              "title": "Legacy dialogue",
              "startNodeId": "start",
              "nodes": [
                {
                  "id": "start",
                  "speakerId": "sakura",
                  "text": "Hello",
                  "choices": [
                    {
                      "text": "Continue",
                      "nextNodeId": "end",
                      "relationshipDelta": 0.1
                    }
                  ]
                },
                { "id": "end", "text": "Done" }
              ]
            }
            """;

        var script = DialogueParser.ParseFromJson(json);

        Assert.NotNull(script);
        Assert.Equal("start", script.StartNodeId);
        Assert.Equal("sakura", script.GetNode("start")?.SpeakerId);
        Assert.Equal(0.1f, script.GetNode("start")?.Choices[0].RelationshipDelta);
    }

    [Fact]
    public void ScriptEngine_DialogueScript_Works()
    {
        var engine = new ScriptEngine();

        // Simulate setting a flag from dialogue
        engine.Execute("setFlag('met_sakura', true)");

        Assert.True(engine.EvaluateCondition("hasFlag('met_sakura')"));
        Assert.False(engine.EvaluateCondition("hasFlag('unknown_flag')"));
    }

    [Fact]
    public void KnowledgeBase_Search_ReturnsRelevantResults()
    {
        var kb = new KnowledgeBase();
        kb.AddEntry(new KnowledgeEntry
        {
            Id = "test",
            Category = "Test",
            Title = "Cherry Blossom Park",
            Content = "A park with beautiful cherry blossoms in spring",
            Tags = ["park", "nature"],
            Importance = 0.9f
        });

        var results = kb.Search("cherry blossom park");
        Assert.NotEmpty(results);
        Assert.Equal("test", results[0].Id);
    }

    [Fact]
    public async Task FullPipeline_CharacterKnowledgeDialogue_EndToEnd()
    {
        // Load all data
        var charManager = new CharacterManager();
        var kb = new KnowledgeBase();
        var dataRoot = Path.Combine(FindProjectRoot(), "data");

        await charManager.LoadCharactersFromDirectory(Path.Combine(dataRoot, "characters"));
        await kb.LoadFromDirectory(Path.Combine(dataRoot, "knowledge"));

        // Create dialogue manager
        var dm = new DialogueManager(charManager, kb);

        // Load and start dialogue
        var script = await DialogueParser.LoadFromFile(Path.Combine(dataRoot, "dialogue", "example_dialogue.json"));
        Assert.NotNull(script);

        var dialogueTexts = new List<string>();
        dm.OnShowDialogue += (name, text) => dialogueTexts.Add($"{name}: {text}");

        dm.StartDialogue(script);

        // Verify first line was shown
        Assert.Single(dialogueTexts);
        Assert.Contains("Sakura", dialogueTexts[0]);

        // Advance and select choice
        dm.Advance();
        dm.SelectChoice(0);

        // Verify more dialogue was shown
        Assert.True(dialogueTexts.Count >= 2);

        // Verify character data was loaded
        var sakura = charManager.GetCharacter("sakura");
        Assert.NotNull(sakura);
        Assert.Equal("happy", sakura.CurrentEmotion);
    }

    private static string FindProjectRoot()
    {
        var dir = AppDomain.CurrentDomain.BaseDirectory;
        for (int i = 0; i < 10; i++)
        {
            if (File.Exists(Path.Combine(dir, "LLMAssistant.sln")))
                return dir;
            if (Directory.Exists(Path.Combine(dir, "data", "characters")))
                return dir;
            var parent = Directory.GetParent(dir);
            if (parent == null) break;
            dir = parent.FullName;
        }
        return AppDomain.CurrentDomain.BaseDirectory;
    }
}
