using LLMAssistant.Core.Services;
using LLMAssistant.Core.Events;
using LLMAssistant.Game.Characters;
using LLMAssistant.Game.Knowledge;

namespace LLMAssistant.AI.Pipeline;

public class InferencePipeline : IGameService
{
    private readonly List<IInferenceEngine> _engines = [];
    private IInferenceEngine? _activeEngine;
    private readonly PromptBuilder _promptBuilder;

    public string ServiceName => "InferencePipeline";
    public IInferenceEngine? ActiveEngine => _activeEngine;
    public IReadOnlyList<IInferenceEngine> Engines => _engines;

    public InferencePipeline()
    {
        _promptBuilder = new PromptBuilder();
    }

    public void RegisterEngine(IInferenceEngine engine)
    {
        _engines.Add(engine);
        _activeEngine ??= engine;
    }

    public void SetActiveEngine(string name)
    {
        _activeEngine = _engines.FirstOrDefault(e => e.Name == name);
    }

    public async Task InitializeEnginesAsync()
    {
        foreach (var engine in _engines)
        {
            var success = await engine.InitializeAsync();
            if (success && _activeEngine == null)
            {
                _activeEngine = engine;
            }
        }
    }

    public async Task<InferenceResult> GenerateResponseAsync(
        string playerInput,
        Character? character = null,
        KnowledgeBase? knowledgeBase = null,
        InferenceOptions? options = null)
    {
        if (_activeEngine == null)
            return InferenceResult.Fail("No inference engine available");

        _promptBuilder.Clear();

        if (character != null)
        {
            _promptBuilder.WithCharacterContext(character);
        }

        if (knowledgeBase != null)
        {
            _promptBuilder.WithKnowledgeContext(knowledgeBase, playerInput);
        }

        _promptBuilder.AddUserMessage(playerInput);

        var prompt = _promptBuilder.Build();

        EventBus.Instance.Publish(new LLMInferenceStartedEvent(prompt));

        var result = await _activeEngine.InferAsync(prompt, options);

        EventBus.Instance.Publish(new LLMInferenceCompletedEvent(
            result.Text, result.DurationMs, result.Success));

        return result;
    }

    public async Task<string> GenerateDialogueAsync(
        string characterId,
        string playerInput,
        CharacterManager characterManager,
        KnowledgeBase knowledgeBase)
    {
        var character = characterManager.GetCharacter(characterId);
        if (character == null) return "";

        var result = await GenerateResponseAsync(playerInput, character, knowledgeBase);

        if (result.Success)
        {
            character.Memory.AddMemory(
                $"Player said: {playerInput}",
                MemoryType.Conversation, 0.5f);
            character.Memory.AddMemory(
                $"Responded: {result.Text}",
                MemoryType.Conversation, 0.5f);
            return result.Text;
        }

        return $"[Error: {result.Error}]";
    }

    public void Initialize() { }
    public void Update(double deltaTime) { }

    public void Shutdown()
    {
        foreach (var engine in _engines)
        {
            engine.Shutdown();
        }
        _engines.Clear();
        _activeEngine = null;
    }
}
