using System.Text.Json;
using LLMAssistant.Assets;

namespace LLMAssistant.Tests;

public class SaveManagerTests
{
    [Fact]
    public void Save_RejectsTraversalSaveIds()
    {
        var root = TempRoot("save_rejects_traversal");
        try
        {
            Directory.CreateDirectory(root);
            var outside = Path.Combine(root, "settings.json");
            File.WriteAllText(outside, "keep me");
            var manager = new SaveManager(Path.Combine(root, "saves"));

            var error = Assert.Throws<ArgumentException>(() => manager.Save(new GameSave
            {
                SaveId = "../settings",
                SaveName = "Bad Save"
            }));

            Assert.Contains("Save id", error.Message);
            Assert.Equal("keep me", File.ReadAllText(outside));
        }
        finally
        {
            RemoveTempRoot(root);
        }
    }

    [Fact]
    public void Load_ReturnsNullForTraversalSaveIds()
    {
        var root = TempRoot("load_rejects_traversal");
        try
        {
            Directory.CreateDirectory(root);
            File.WriteAllText(Path.Combine(root, "settings.json"), JsonSerializer.Serialize(new GameSave
            {
                SaveId = "settings",
                SaveName = "External"
            }));
            var manager = new SaveManager(Path.Combine(root, "saves"));

            var save = manager.Load("../settings");

            Assert.Null(save);
        }
        finally
        {
            RemoveTempRoot(root);
        }
    }

    [Fact]
    public void DeleteSave_IgnoresTraversalSaveIds()
    {
        var root = TempRoot("delete_rejects_traversal");
        try
        {
            Directory.CreateDirectory(root);
            var outside = Path.Combine(root, "settings.json");
            File.WriteAllText(outside, "keep me");
            var manager = new SaveManager(Path.Combine(root, "saves"));

            manager.DeleteSave("../settings");

            Assert.True(File.Exists(outside));
            Assert.Equal("keep me", File.ReadAllText(outside));
        }
        finally
        {
            RemoveTempRoot(root);
        }
    }

    [Fact]
    public void GetAllSaves_IgnoresInvalidOrMismatchedSaveIds()
    {
        var root = TempRoot("list_filters_ids");
        try
        {
            var savesDir = Path.Combine(root, "saves");
            Directory.CreateDirectory(savesDir);
            var manager = new SaveManager(savesDir);
            manager.Save(new GameSave
            {
                SaveId = "slot_1",
                SaveName = "Good Save"
            });
            File.WriteAllText(Path.Combine(savesDir, "evil.json"), JsonSerializer.Serialize(new GameSave
            {
                SaveId = "../settings",
                SaveName = "Bad Save"
            }));
            File.WriteAllText(Path.Combine(savesDir, "mismatch.json"), JsonSerializer.Serialize(new GameSave
            {
                SaveId = "other_slot",
                SaveName = "Mismatched Save"
            }));

            var saves = manager.GetAllSaves();

            var save = Assert.Single(saves);
            Assert.Equal("slot_1", save.SaveId);
        }
        finally
        {
            RemoveTempRoot(root);
        }
    }

    private static string TempRoot(string label)
    {
        return Path.Combine(
            Path.GetTempPath(),
            $"monogatari_save_manager_{label}_{DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()}_{Guid.NewGuid():N}");
    }

    private static void RemoveTempRoot(string root)
    {
        if (Directory.Exists(root))
        {
            Directory.Delete(root, true);
        }
    }
}
