using System.Text.Json;
using LLMAssistant.Core.Services;
using LLMAssistant.Core;
using LLMAssistant.Core.Events;
using LLMAssistant.Game.Characters;
using LLMAssistant.Game.Dialogue;

namespace LLMAssistant.Assets;

public class SaveManager : IGameService
{
    private readonly string _saveDirectory;

    public string ServiceName => "SaveManager";

    public SaveManager(string saveDirectory = "saves")
    {
        _saveDirectory = saveDirectory;
    }

    public void Save(GameSave save)
    {
        Directory.CreateDirectory(_saveDirectory);

        var path = Path.Combine(_saveDirectory, $"{save.SaveId}.json");
        var json = JsonSerializer.Serialize(save, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(path, json);

        EventBus.Instance.Publish(new GameSaveEvent(path));
    }

    public GameSave? Load(string saveId)
    {
        var path = Path.Combine(_saveDirectory, $"{saveId}.json");
        if (!File.Exists(path)) return null;

        var json = File.ReadAllText(path);
        var save = JsonSerializer.Deserialize<GameSave>(json, JsonOptions.Default);

        if (save != null)
        {
            EventBus.Instance.Publish(new GameLoadEvent(path));
        }

        return save;
    }

    public List<GameSave> GetAllSaves()
    {
        if (!Directory.Exists(_saveDirectory)) return [];

        return Directory.GetFiles(_saveDirectory, "*.json")
            .Select(f =>
            {
                try
                {
                    var json = File.ReadAllText(f);
                    return JsonSerializer.Deserialize<GameSave>(json, JsonOptions.Default);
                }
                catch
                {
                    return null;
                }
            })
            .Where(s => s != null)
            .Cast<GameSave>()
            .OrderByDescending(s => s.Timestamp)
            .ToList();
    }

    public void DeleteSave(string saveId)
    {
        var path = Path.Combine(_saveDirectory, $"{saveId}.json");
        if (File.Exists(path))
        {
            File.Delete(path);
        }
    }

    public GameSave CreateSaveFromState(
        string sceneName,
        DialogueManager? dialogueManager,
        CharacterManager? characterManager,
        Dictionary<string, string> variables)
    {
        var save = new GameSave
        {
            SaveName = $"Save {DateTime.Now:yyyy-MM-dd HH:mm:ss}",
            CurrentScene = sceneName,
            Variables = new Dictionary<string, string>(variables)
        };

        if (dialogueManager?.CurrentScript != null)
        {
            save.CurrentDialogueId = dialogueManager.CurrentScript.Id;
            save.CurrentNodeId = dialogueManager.CurrentNode?.Id ?? "";
        }

        if (characterManager != null)
        {
            foreach (var (id, character) in characterManager.Characters)
            {
                save.Characters[id] = new CharacterSaveData
                {
                    CurrentEmotion = character.CurrentEmotion,
                    Relationships = new Dictionary<string, float>(character.Relationship),
                    MemoryContents = character.Memory.Memories.Select(m => m.Content).ToList()
                };
            }
        }

        return save;
    }

    public void Initialize() { }
    public void Update(double deltaTime) { }
    public void Shutdown() { }
}
