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
        _basePath = Path.GetFullPath(string.IsNullOrWhiteSpace(path) ? "." : path);
    }

    public string ResolvePath(string relativePath)
    {
        return SafeAssetPath(relativePath);
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
        var fullPath = TryResolvePath(relativePath);
        if (fullPath == null) return default;
        if (!File.Exists(fullPath)) return default;

        var json = await File.ReadAllTextAsync(fullPath);
        return JsonSerializer.Deserialize<T>(json, JsonOptions.Default);
    }

    public string? LoadText(string relativePath)
    {
        var fullPath = TryResolvePath(relativePath);
        if (fullPath == null) return null;
        return File.Exists(fullPath) ? File.ReadAllText(fullPath) : null;
    }

    public async Task<string?> LoadTextAsync(string relativePath)
    {
        var fullPath = TryResolvePath(relativePath);
        if (fullPath == null) return null;
        return File.Exists(fullPath) ? await File.ReadAllTextAsync(fullPath) : null;
    }

    public byte[]? LoadBytes(string relativePath)
    {
        var fullPath = TryResolvePath(relativePath);
        if (fullPath == null) return null;
        return File.Exists(fullPath) ? File.ReadAllBytes(fullPath) : null;
    }

    public void Initialize() { }
    public void Update(double deltaTime) { }
    public void Shutdown() { _assets.Clear(); }

    private string? TryResolvePath(string relativePath)
    {
        try
        {
            return SafeAssetPath(relativePath);
        }
        catch (ArgumentException)
        {
            return null;
        }
    }

    private string SafeAssetPath(string relativePath)
    {
        var normalizedRelative = NormalizeAssetRelativePath(relativePath);
        var root = Path.GetFullPath(string.IsNullOrWhiteSpace(_basePath) ? "." : _basePath);
        var path = Path.GetFullPath(Path.Combine(root, normalizedRelative));
        var rootPrefix = root.EndsWith(Path.DirectorySeparatorChar)
            ? root
            : root + Path.DirectorySeparatorChar;

        if (!path.StartsWith(rootPrefix, OperatingSystem.IsWindows()
            ? StringComparison.OrdinalIgnoreCase
            : StringComparison.Ordinal))
        {
            throw new ArgumentException("Asset path must stay inside the asset root.", nameof(relativePath));
        }

        return path;
    }

    private static string NormalizeAssetRelativePath(string relativePath)
    {
        if (string.IsNullOrEmpty(relativePath) || relativePath.Any(char.IsControl))
        {
            throw new ArgumentException(
                "Asset paths must be non-empty and cannot contain control characters.",
                nameof(relativePath));
        }

        var normalized = relativePath
            .Replace('\\', Path.DirectorySeparatorChar)
            .Replace('/', Path.DirectorySeparatorChar);

        if (normalized.Contains(':') || Path.IsPathRooted(normalized))
        {
            throw new ArgumentException(
                "Asset paths cannot be absolute or contain drive prefixes or URI schemes.",
                nameof(relativePath));
        }

        var segments = normalized.Split(Path.DirectorySeparatorChar);
        if (segments.Any(segment => segment.Length == 0 || segment is "." or ".."))
        {
            throw new ArgumentException(
                "Asset paths cannot contain empty, current, or parent directory segments.",
                nameof(relativePath));
        }

        return normalized;
    }
}
