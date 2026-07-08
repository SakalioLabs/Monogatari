using LLMAssistant.Assets;

namespace LLMAssistant.Tests;

public class AssetManagerTests
{
    [Fact]
    public void ResolvePath_RejectsTraversalAssetPaths()
    {
        var root = TempRoot("resolve_rejects_traversal");
        try
        {
            var assetsDir = Path.Combine(root, "assets");
            Directory.CreateDirectory(assetsDir);
            File.WriteAllText(Path.Combine(root, "settings.json"), "keep me");
            var manager = AssetManagerFor(assetsDir);

            var error = Assert.Throws<ArgumentException>(() => manager.ResolvePath("../settings.json"));

            Assert.Contains("Asset path", error.Message);
        }
        finally
        {
            RemoveTempRoot(root);
        }
    }

    [Fact]
    public void LoadText_ReturnsNullForEscapingAssetPaths()
    {
        var root = TempRoot("load_text_rejects_traversal");
        try
        {
            var assetsDir = Path.Combine(root, "assets");
            Directory.CreateDirectory(assetsDir);
            File.WriteAllText(Path.Combine(root, "settings.json"), "outside");
            var manager = AssetManagerFor(assetsDir);

            var content = manager.LoadText("../settings.json");

            Assert.Null(content);
        }
        finally
        {
            RemoveTempRoot(root);
        }
    }

    [Fact]
    public void LoadBytes_ReturnsNullForAbsoluteAssetPaths()
    {
        var root = TempRoot("load_bytes_rejects_absolute");
        try
        {
            var assetsDir = Path.Combine(root, "assets");
            Directory.CreateDirectory(assetsDir);
            var outside = Path.Combine(root, "secret.bin");
            File.WriteAllBytes(outside, [1, 2, 3]);
            var manager = AssetManagerFor(assetsDir);

            var bytes = manager.LoadBytes(outside);

            Assert.Null(bytes);
        }
        finally
        {
            RemoveTempRoot(root);
        }
    }

    [Fact]
    public async Task LoadJsonAsync_ReturnsNullForUriLikeAssetPaths()
    {
        var root = TempRoot("load_json_rejects_uri");
        try
        {
            var assetsDir = Path.Combine(root, "assets");
            Directory.CreateDirectory(assetsDir);
            var manager = AssetManagerFor(assetsDir);

            var value = await manager.LoadJsonAsync<Dictionary<string, string>>("https://example.test/asset.json");

            Assert.Null(value);
        }
        finally
        {
            RemoveTempRoot(root);
        }
    }

    [Fact]
    public void LoadText_AllowsNestedProjectAssetPaths()
    {
        var root = TempRoot("load_text_allows_nested");
        try
        {
            var assetsDir = Path.Combine(root, "assets");
            var characterDir = Path.Combine(assetsDir, "characters");
            Directory.CreateDirectory(characterDir);
            File.WriteAllText(Path.Combine(characterDir, "sakura.txt"), "hello");
            var manager = AssetManagerFor(assetsDir);

            var content = manager.LoadText("characters\\sakura.txt");
            var resolved = manager.ResolvePath("characters/sakura.txt");

            Assert.Equal("hello", content);
            Assert.EndsWith(Path.Combine("characters", "sakura.txt"), resolved);
        }
        finally
        {
            RemoveTempRoot(root);
        }
    }

    private static AssetManager AssetManagerFor(string basePath)
    {
        var manager = new AssetManager();
        manager.SetBasePath(basePath);
        return manager;
    }

    private static string TempRoot(string label)
    {
        return Path.Combine(
            Path.GetTempPath(),
            $"monogatari_asset_manager_{label}_{DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()}_{Guid.NewGuid():N}");
    }

    private static void RemoveTempRoot(string root)
    {
        if (Directory.Exists(root))
        {
            Directory.Delete(root, true);
        }
    }
}
