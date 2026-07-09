using LLMAssistant.Core.Services;
using LLMAssistant.Core.Events;
using LLMAssistant.Game.Characters;
using LLMAssistant.Game.Knowledge;

namespace LLMAssistant.Game.Dialogue;

public class DialogueManager : IGameService
{
    private DialogueScript? _currentScript;
    private DialogueNode? _currentNode;
    private readonly CharacterManager _characterManager;
    private readonly KnowledgeBase _knowledgeBase;

    public string ServiceName => "DialogueManager";
    public DialogueScript? CurrentScript => _currentScript;
    public DialogueNode? CurrentNode => _currentNode;
    public bool IsActive => _currentScript != null;

    public event Action<string, string>? OnShowDialogue;
    public event Action<IReadOnlyList<string>>? OnShowChoices;
    public event Action? OnDialogueEnd;

    // LLM callback - set by AI module
    public Func<string, string?, Task<string>>? LLMInferenceCallback { get; set; }

    public DialogueManager(CharacterManager characterManager, KnowledgeBase knowledgeBase)
    {
        _characterManager = characterManager;
        _knowledgeBase = knowledgeBase;
    }

    public void StartDialogue(DialogueScript script)
    {
        _currentScript = script;
        var startNode = script.GetNode(script.StartNodeId);
        if (startNode != null)
        {
            ProcessNode(startNode);
        }
        EventBus.Instance.Publish(new DialogueStartedEvent(script.Title));
    }

    public void Advance()
    {
        if (_currentNode == null || _currentScript == null) return;
        if (_currentNode.Choices.Count > 0) return;

        if (_currentNode.NextNodeId != null)
        {
            var nextNode = _currentScript.GetNode(_currentNode.NextNodeId);
            if (nextNode != null)
            {
                ProcessNode(nextNode);
            }
            else
            {
                EndDialogue();
            }
        }
        else
        {
            EndDialogue();
        }
    }

    public void SelectChoice(int index)
    {
        if (_currentNode == null || index < 0 || index >= _currentNode.Choices.Count) return;

        var choice = _currentNode.Choices[index];

        foreach (var (characterId, delta) in choice.RelationshipChanges)
        {
            var character = _characterManager.GetCharacter(characterId);
            character?.AdjustRelationship("player", delta);
        }

        if (choice.RelationshipChanges.Count == 0 &&
            choice.RelationshipDelta.HasValue &&
            _currentNode.SpeakerId != null)
        {
            var character = _characterManager.GetCharacter(_currentNode.SpeakerId);
            character?.AdjustRelationship("player", choice.RelationshipDelta.Value);
        }

        EventBus.Instance.Publish(new ChoiceMadeEvent(index, choice.Text));

        if (choice.NextNodeId != null)
        {
            var nextNode = _currentScript!.GetNode(choice.NextNodeId);
            if (nextNode != null)
            {
                ProcessNode(nextNode);
            }
            else
            {
                EndDialogue();
            }
        }
        else
        {
            Advance();
        }
    }

    private async void ProcessNode(DialogueNode node)
    {
        _currentNode = node;

        // Update character emotion
        if (node.Emotion != null && node.SpeakerId != null)
        {
            var character = _characterManager.GetCharacter(node.SpeakerId);
            character?.UpdateEmotion(node.Emotion);
            EventBus.Instance.Publish(new CharacterEmotionChangedEvent(node.SpeakerId, node.Emotion));
        }

        // Handle LLM-generated content
        var displayText = node.Text;
        if (node.UseLLM && LLMInferenceCallback != null)
        {
            try
            {
                var context = node.SpeakerId != null
                    ? _characterManager.GetCharacter(node.SpeakerId)?.BuildSystemPrompt() ?? ""
                    : "";
                var systemPrompt = node.LLMSystemPrompt ?? context;
                displayText = await LLMInferenceCallback(node.LLMPrompt ?? node.Text, systemPrompt);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"LLM inference failed: {ex.Message}");
                displayText = node.Text;
            }
        }

        // Display dialogue
        var speakerName = "";
        if (node.SpeakerId != null)
        {
            var speaker = _characterManager.GetCharacter(node.SpeakerId);
            speakerName = speaker?.DisplayName ?? node.SpeakerId;
        }

        OnShowDialogue?.Invoke(speakerName, displayText);

        // Show choices if available
        if (node.Choices.Count > 0)
        {
            var choiceTexts = node.Choices.Select(c => c.Text).ToList().AsReadOnly();
            OnShowChoices?.Invoke(choiceTexts);
        }
    }

    private void EndDialogue()
    {
        var speaker = _currentNode?.SpeakerId;
        _currentScript = null;
        _currentNode = null;
        OnDialogueEnd?.Invoke();
        EventBus.Instance.Publish(new DialogueEndedEvent(speaker ?? ""));
    }

    public void Initialize() { }
    public void Update(double deltaTime) { }
    public void Shutdown() { }
}
