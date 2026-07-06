namespace LLMAssistant.Core.Events;

public record DialogueStartedEvent(string CharacterName);
public record DialogueEndedEvent(string CharacterName);
public record ChoiceMadeEvent(int ChoiceIndex, string ChoiceText);
public record SceneChangedEvent(string SceneName);
public record CharacterEmotionChangedEvent(string CharacterName, string Emotion);
public record KnowledgeQueriedEvent(string Query, List<string> Results);
public record LLMResponseEvent(string Response, double DurationMs);
public record GameSaveEvent(string SavePath);
public record GameLoadEvent(string SavePath);
public record LLMInferenceStartedEvent(string Prompt);
public record LLMInferenceCompletedEvent(string Response, double DurationMs, bool Success);
