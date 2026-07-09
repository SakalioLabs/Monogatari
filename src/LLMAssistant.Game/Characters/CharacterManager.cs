using System.Text.Json;
using LLMAssistant.Core;
using LLMAssistant.Core.Services;

namespace LLMAssistant.Game.Characters;

public class CharacterManager : IGameService
{
    private readonly Dictionary<string, Character> _characters = new();

    public string ServiceName => "CharacterManager";
    public IReadOnlyDictionary<string, Character> Characters => _characters;

    public void RegisterCharacter(Character character)
    {
        _characters[character.Id] = character;
    }

    public Character? GetCharacter(string id)
    {
        return _characters.TryGetValue(id, out var character) ? character : null;
    }

    public List<Character> GetAllCharacters()
    {
        return _characters.Values.ToList();
    }

    public async Task LoadCharactersFromDirectory(string directoryPath)
    {
        if (!Directory.Exists(directoryPath)) return;

        foreach (var file in Directory.GetFiles(directoryPath, "*.json"))
        {
            try
            {
                var json = await File.ReadAllTextAsync(file);
                using var document = JsonDocument.Parse(json);
                var character = JsonSerializer.Deserialize<Character>(json, JsonOptions.Default);
                if (character != null)
                {
                    ApplyCurrentCharacterJson(document.RootElement, character);
                    if (!string.IsNullOrEmpty(character.PersonalityJson))
                    {
                        character.Personality = JsonSerializer.Deserialize<Personality>(character.PersonalityJson, JsonOptions.Default)
                            ?? new Personality();
                    }
                    RegisterCharacter(character);
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to load character from {file}: {ex.Message}");
            }
        }
    }

    private static void ApplyCurrentCharacterJson(JsonElement root, Character character)
    {
        if (TryGetString(root, "displayName", out var displayName)
            || TryGetString(root, "display_name", out displayName)
            || TryGetString(root, "name", out displayName))
        {
            character.DisplayName = displayName;
        }

        if (TryGetString(root, "currentEmotion", out var emotion)
            || TryGetString(root, "current_emotion", out emotion)
            || TryGetString(root, "emotion", out emotion)
            || TryGetNestedString(root, "emotions", "default", out emotion))
        {
            character.CurrentEmotion = emotion;
        }

        if (TryGetString(root, "spritePath", out var spritePath)
            || TryGetString(root, "sprite_path", out spritePath))
        {
            character.SpritePath = spritePath;
        }

        if (TryGetProperty(root, "personality", out var personality)
            && personality.ValueKind == JsonValueKind.Object)
        {
            ApplyCurrentPersonalityJson(personality, character.Personality);
        }

        if (string.IsNullOrWhiteSpace(character.DisplayName))
        {
            character.DisplayName = !string.IsNullOrWhiteSpace(character.Name)
                ? character.Name
                : character.Id;
        }
    }

    private static void ApplyCurrentPersonalityJson(JsonElement source, Personality personality)
    {
        foreach (var trait in new[] { "openness", "conscientiousness", "extraversion", "agreeableness", "neuroticism" })
        {
            if (TryGetFloat(source, trait, out var value))
            {
                personality.Traits[trait] = value;
            }
        }

        if (TryGetString(source, "speechStyle", out var speechStyle)
            || TryGetString(source, "speech_style", out speechStyle))
        {
            personality.SpeechStyle = speechStyle;
        }

        if (TryGetString(source, "description", out var description))
        {
            personality.Description = description;
        }

        if (TryGetStringArray(source, "likes", out var likes))
        {
            personality.Likes = likes;
        }

        if (TryGetStringArray(source, "dislikes", out var dislikes))
        {
            personality.Dislikes = dislikes;
        }
    }

    private static bool TryGetNestedString(JsonElement source, string objectName, string propertyName, out string value)
    {
        value = "";
        return TryGetProperty(source, objectName, out var nested)
            && nested.ValueKind == JsonValueKind.Object
            && TryGetString(nested, propertyName, out value);
    }

    private static bool TryGetString(JsonElement source, string propertyName, out string value)
    {
        value = "";
        if (!TryGetProperty(source, propertyName, out var property) || property.ValueKind != JsonValueKind.String)
        {
            return false;
        }

        value = property.GetString() ?? "";
        return !string.IsNullOrWhiteSpace(value);
    }

    private static bool TryGetFloat(JsonElement source, string propertyName, out float value)
    {
        value = 0;
        if (!TryGetProperty(source, propertyName, out var property) || property.ValueKind != JsonValueKind.Number)
        {
            return false;
        }

        return property.TryGetSingle(out value);
    }

    private static bool TryGetStringArray(JsonElement source, string propertyName, out List<string> values)
    {
        values = [];
        if (!TryGetProperty(source, propertyName, out var property) || property.ValueKind != JsonValueKind.Array)
        {
            return false;
        }

        values = property
            .EnumerateArray()
            .Where(item => item.ValueKind == JsonValueKind.String)
            .Select(item => item.GetString())
            .Where(item => !string.IsNullOrWhiteSpace(item))
            .Select(item => item!)
            .ToList();
        return true;
    }

    private static bool TryGetProperty(JsonElement source, string propertyName, out JsonElement value)
    {
        if (source.TryGetProperty(propertyName, out value))
        {
            return true;
        }

        foreach (var property in source.EnumerateObject())
        {
            if (string.Equals(property.Name, propertyName, StringComparison.OrdinalIgnoreCase))
            {
                value = property.Value;
                return true;
            }
        }

        value = default;
        return false;
    }

    public void Initialize() { }
    public void Update(double deltaTime) { }
    public void Shutdown() { }
}
