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
                var character = JsonSerializer.Deserialize<Character>(json, JsonOptions.Default);
                if (character != null)
                {
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

    public void Initialize() { }
    public void Update(double deltaTime) { }
    public void Shutdown() { }
}
