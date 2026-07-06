using System.Text.Json;
using LLMAssistant.Core;
using LLMAssistant.Core.Services;

namespace LLMAssistant.Assets;

public class AssetManager : IGameService
{
    private readonly Dictionary<string, object> _assets = new();
    private string _basePath = "";

    public string ServiceName => "AssetManager";
    public string BasePath => _basePath;

    public void SetBasePath(string path)
    {
        _basePath = path;
    }

    public string ResolvePath(string relativePath)
    {
        return Path.Combine(_basePath, relativePath);
    }

    public void RegisterAsset<T>(string key, T asset) where T : class
    {
        _assets[key] = asset;
    }

    public T? GetAsset<T>(string key) where T : class
    {
        return _assets.TryGetValue(key, out var asset) ? asset as T : null;
    }

    public async Task<T?> LoadJsonAsync<T>(string relativePath)
    {
        var fullPath = ResolvePath(relativePath);
        if (!File.Exists(fullPath)) return default;

        var json = await File.ReadAllTextAsync(fullPath);
        return JsonSerializer.Deserialize<T>(json, JsonOptions.Default);
    }

    public string? LoadText(string relativePath)
    {
        var fullPath = ResolvePath(relativePath);
        return File.Exists(fullPath) ? File.ReadAllText(fullPath) : null;
    }

    public async Task<string?> LoadTextAsync(string relativePath)
    {
        var fullPath = ResolvePath(relativePath);
        return File.Exists(fullPath) ? await File.ReadAllTextAsync(fullPath) : null;
    }

    public byte[]? LoadBytes(string relativePath)
    {
        var fullPath = ResolvePath(relativePath);
        return File.Exists(fullPath) ? File.ReadAllBytes(fullPath) : null;
    }

    public void Initialize() { }
    public void Update(double deltaTime) { }
    public void Shutdown() { _assets.Clear(); }
}
