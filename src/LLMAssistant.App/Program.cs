using System.Text.Json;
using LLMAssistant.Core;
using LLMAssistant.Core.Services;
using LLMAssistant.Renderer;
using LLMAssistant.Renderer.SDL2;
using LLMAssistant.Renderer.UI;
using LLMAssistant.Game.Scenes;
using LLMAssistant.Game.Dialogue;
using LLMAssistant.Game.Characters;
using LLMAssistant.Game.Knowledge;
using LLMAssistant.AI;
using LLMAssistant.AI.API;
using LLMAssistant.AI.ONNX;
using LLMAssistant.AI.Pipeline;
using LLMAssistant.Scripting;
using LLMAssistant.Assets;
using static LLMAssistant.Renderer.SDL2.SDL2Native;

namespace LLMAssistant.App;

class Program
{
    static async Task Main(string[] args)
    {
        Console.WriteLine("=== LLM Galgame Engine ===");
        Console.WriteLine("Initializing...");

        // Load settings - try multiple paths
        var basePath = FindProjectRoot();
        var settingsPath = Path.Combine(basePath, "data", "settings.json");
        if (!File.Exists(settingsPath))
            settingsPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "data", "settings.json");

        Settings? settings = null;
        if (File.Exists(settingsPath))
        {
            var json = File.ReadAllText(settingsPath);
            settings = JsonSerializer.Deserialize<Settings>(json, LLMAssistant.Core.JsonOptions.Default);
        }

        var engineTitle = settings?.Engine?.Title ?? "LLM Galgame Engine";
        var engineWidth = settings?.Engine?.Width ?? 1280;
        var engineHeight = settings?.Engine?.Height ?? 720;

        // Initialize engine core
        var engine = new Engine();

        // Initialize SDL2 window
        var window = new WindowManager(engineTitle, engineWidth, engineHeight);
        if (!window.Initialize())
        {
            Console.WriteLine("Failed to initialize SDL2 window.");
            return;
        }

        var renderContext = new RenderContext(window.Window);
        var textRenderer = new TextRenderer(renderContext);

        // Register renderer services
        engine.Services.Register(window);
        engine.Services.Register(renderContext);
        engine.Services.Register(textRenderer);
        textRenderer.LoadFont("default", FindFont("arial.ttf"), 18);
        textRenderer.LoadFont("title", FindFont("arial.ttf"), 36);

        // Initialize game systems
        var characterManager = new CharacterManager();
        var knowledgeBase = new KnowledgeBase();
        var dialogueManager = new DialogueManager(characterManager, knowledgeBase);
        var sceneManager = new SceneManager();
        var scriptEngine = new ScriptEngine();
        var uiManager = new UIManager(renderContext);
        var assetManager = new AssetManager();
        var saveManager = new SaveManager();
        var inferencePipeline = new InferencePipeline();

        // Register all services
        engine.RegisterService(characterManager);
        engine.RegisterService(knowledgeBase);
        engine.RegisterService(dialogueManager);
        engine.RegisterService(sceneManager);
        engine.RegisterService(scriptEngine);
        engine.RegisterService(uiManager);
        engine.RegisterService(assetManager);
        engine.RegisterService(saveManager);
        engine.RegisterService(inferencePipeline);

        // Load game data
        assetManager.SetBasePath(basePath);

        await characterManager.LoadCharactersFromDirectory(
            Path.Combine(basePath, "data", "characters"));
        await knowledgeBase.LoadFromDirectory(
            Path.Combine(basePath, "data", "knowledge"));

        Console.WriteLine($"Loaded {characterManager.Characters.Count} characters");
        Console.WriteLine($"Loaded {knowledgeBase.Entries.Count} knowledge entries");

        // Set up LLM integration
        var aiProvider = settings?.AI?.Provider ?? "api";
        if (aiProvider == "onnx" && settings?.AI?.ONNX != null)
        {
            var onnxConfig = new ModelConfig
            {
                ModelPath = settings.AI.ONNX.ModelPath,
                TokenizerPath = settings.AI.ONNX.TokenizerPath,
                UseDirectML = settings.AI.ONNX.UseDirectML,
                MaxSequenceLength = settings.AI.ONNX.MaxSequenceLength
            };
            inferencePipeline.RegisterEngine(new ONNXEngine(onnxConfig));
        }
        else if (settings?.AI?.API != null)
        {
            var apiConfig = new APIConfig
            {
                BaseUrl = settings.AI.API.BaseUrl,
                ApiKey = settings.AI.API.ApiKey,
                Model = settings.AI.API.Model,
                MaxTokens = settings.AI.API.MaxTokens,
                Temperature = settings.AI.API.Temperature
            };
            inferencePipeline.RegisterEngine(new APIEngine(apiConfig));
        }

        // Initialize LLM engines
        await inferencePipeline.InitializeEnginesAsync();

        // Connect LLM to dialogue system
        dialogueManager.LLMInferenceCallback = async (prompt, systemPrompt) =>
        {
            var result = await inferencePipeline.ActiveEngine!.InferAsync(
                $"{systemPrompt}\n\nUser: {prompt}\n\nAssistant:");
            return result.Success ? result.Text : $"[LLM Error: {result.Error}]";
        };

        // Register scenes
        sceneManager.RegisterScene("Title", () => new TitleScene());
        sceneManager.RegisterScene("Game", () => new GameScene());
        sceneManager.RegisterScene("Settings", () => new SettingsScene());

        // Start engine
        engine.Initialize();
        sceneManager.PushScene("Title");

        Console.WriteLine("Engine ready. Starting game loop...");

        // Main game loop
        bool running = true;
        while (running && engine.IsRunning)
        {
            // Handle events
            while (SDL_PollEvent(out var evt) != 0)
            {
                switch (evt.type)
                {
                    case SDL_EventType.SDL_QUIT:
                        running = false;
                        break;

                    case SDL_EventType.SDL_WINDOWEVENT:
                        if (evt.GetWindowEvent() == 5) // RESIZED
                        {
                            window.SetSize(evt.GetWindowData1(), evt.GetWindowData2());
                        }
                        break;

                    default:
                        sceneManager.HandleInput(evt);
                        break;
                }
            }

            // Update
            engine.Update();

            // Draw
            renderContext.Clear(0, 0, 0);
            sceneManager.Draw(renderContext);
            uiManager.Draw();
            renderContext.Present();

            // Small delay to prevent 100% CPU
            SDL_Delay(1);
        }

        // Cleanup
        engine.Shutdown();
        textRenderer.Dispose();
        renderContext.Dispose();
        window.Dispose();

        Console.WriteLine("Engine shut down cleanly.");
    }

    static string FindFont(string fontName)
    {
        var projectRoot = FindProjectRoot();
        // Try system fonts
        var systemPaths = new[]
        {
            Path.Combine(projectRoot, "assets", "fonts", fontName),
            Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Fonts), fontName),
            fontName
        };

        foreach (var path in systemPaths)
        {
            if (File.Exists(path)) return path;
        }

        // Fallback to any available font
        var fontsDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Fonts));
        var arial = Path.Combine(fontsDir, "arial.ttf");
        if (File.Exists(arial)) return arial;

        var msyh = Path.Combine(fontsDir, "msyh.ttc");
        if (File.Exists(msyh)) return msyh;

        return fontName; // Let SDL_ttf try
    }

    static string FindProjectRoot()
    {
        // When running via dotnet run, base is bin/Debug/net9.0/
        var dir = AppDomain.CurrentDomain.BaseDirectory;
        for (int i = 0; i < 10; i++)
        {
            if (File.Exists(Path.Combine(dir, "LLMAssistant.sln")))
                return dir;
            if (Directory.Exists(Path.Combine(dir, "data")))
                return dir;
            var parent = Directory.GetParent(dir);
            if (parent == null) break;
            dir = parent.FullName;
        }
        return AppDomain.CurrentDomain.BaseDirectory;
    }
}

// Settings model classes
class Settings
{
    public EngineSettings? Engine { get; set; }
    public RenderSettings? Rendering { get; set; }
    public DialogueSettings? Dialogue { get; set; }
    public AISettings? AI { get; set; }
    public PathSettings? Paths { get; set; }
}

class EngineSettings
{
    public string Title { get; set; } = "LLM Galgame Engine";
    public int Width { get; set; } = 1280;
    public int Height { get; set; } = 720;
    public int TargetFps { get; set; } = 60;
    public bool Fullscreen { get; set; }
}

class RenderSettings
{
    public bool Vsync { get; set; } = true;
    public string DefaultFont { get; set; } = "default";
    public int FontSize { get; set; } = 18;
    public int TitleFontSize { get; set; } = 36;
}

class DialogueSettings
{
    public double TypewriterSpeed { get; set; } = 30;
    public double AutoAdvanceDelay { get; set; }
    public int DefaultBackgroundAlpha { get; set; } = 200;
}

class AISettings
{
    public string Provider { get; set; } = "api";
    public APISettings? API { get; set; }
    public ONNXSettings? ONNX { get; set; }
}

class APISettings
{
    public string BaseUrl { get; set; } = "https://api.openai.com/v1";
    public string ApiKey { get; set; } = "";
    public string Model { get; set; } = "gpt-3.5-turbo";
    public int MaxTokens { get; set; } = 512;
    public float Temperature { get; set; } = 0.7f;
}

class ONNXSettings
{
    public string ModelPath { get; set; } = "";
    public string TokenizerPath { get; set; } = "";
    public bool UseDirectML { get; set; } = true;
    public int MaxSequenceLength { get; set; } = 2048;
}

class PathSettings
{
    public string Characters { get; set; } = "data/characters";
    public string Knowledge { get; set; } = "data/knowledge";
    public string Dialogue { get; set; } = "data/dialogue";
    public string Assets { get; set; } = "assets";
    public string Saves { get; set; } = "saves";
}
