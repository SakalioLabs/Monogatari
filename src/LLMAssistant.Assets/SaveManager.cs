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

        var path = SafeSavePath(save.SaveId);
        var json = JsonSerializer.Serialize(save, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(path, json);

        EventBus.Instance.Publish(new GameSaveEvent(path));
    }

    public GameSave? Load(string saveId)
    {
        if (!IsValidSaveId(saveId)) return null;

        var path = SafeSavePath(saveId);
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
            .Where(f => IsValidSaveId(Path.GetFileNameWithoutExtension(f)))
            .Select(f =>
            {
                try
                {
                    var json = File.ReadAllText(f);
                    var save = JsonSerializer.Deserialize<GameSave>(json, JsonOptions.Default);
                    var fileSaveId = Path.GetFileNameWithoutExtension(f);
                    return save != null && save.SaveId == fileSaveId && IsValidSaveId(save.SaveId)
                        ? save
                        : null;
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
        if (!IsValidSaveId(saveId)) return;

        var path = SafeSavePath(saveId);
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

    private string SafeSavePath(string saveId)
    {
        if (!IsValidSaveId(saveId))
        {
            throw new ArgumentException(
                "Save id cannot contain path separators, dots, whitespace, or control characters.",
                nameof(saveId));
        }

        var root = Path.GetFullPath(_saveDirectory);
        var path = Path.GetFullPath(Path.Combine(root, $"{saveId}.json"));
        var rootPrefix = root.EndsWith(Path.DirectorySeparatorChar)
            ? root
            : root + Path.DirectorySeparatorChar;

        if (!path.StartsWith(rootPrefix, OperatingSystem.IsWindows()
            ? StringComparison.OrdinalIgnoreCase
            : StringComparison.Ordinal))
        {
            throw new ArgumentException("Save path must stay inside the save directory.", nameof(saveId));
        }

        return path;
    }

    private static bool IsValidSaveId(string saveId)
    {
        return !string.IsNullOrEmpty(saveId)
            && saveId.Length <= 128
            && saveId.All(ch => char.IsAsciiLetterOrDigit(ch) || ch is '-' or '_');
    }
}
