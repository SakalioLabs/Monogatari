using System.Numerics;
using LLMAssistant.Core;
using LLMAssistant.Core.Services;
using LLMAssistant.Renderer;
using LLMAssistant.Renderer.UI;
using LLMAssistant.Renderer.SDL2;
using static LLMAssistant.Renderer.SDL2.SDL2Native;

namespace LLMAssistant.Game.Scenes;

public class TitleScene : IScene
{
    public string Name => "Title";
    public bool IsOpaque { get; set; } = true;

    private Button _startButton = null!;
    private Button _settingsButton = null!;
    private Button _exitButton = null!;
    private SceneManager _sceneManager = null!;

    public void Enter()
    {
        _sceneManager = ServiceLocator.Instance.GetRequired<SceneManager>();

        _startButton = new Button
        {
            Text = "Start Game",
            Size = new Vector2(200, 50),
            Position = new Vector2(540, 350)
        };
        _startButton.OnClick += () => _sceneManager.PushScene("Game");

        _settingsButton = new Button
        {
            Text = "Settings",
            Size = new Vector2(200, 50),
            Position = new Vector2(540, 420)
        };
        _settingsButton.OnClick += () => _sceneManager.PushScene("Settings");

        _exitButton = new Button
        {
            Text = "Exit",
            Size = new Vector2(200, 50),
            Position = new Vector2(540, 490)
        };
        _exitButton.OnClick += () =>
        {
            var engine = ServiceLocator.Instance.GetRequired<Engine>();
            engine.Stop();
        };
    }

    public void Exit() { }
    public void Pause() { }
    public void Resume() { }

    public void Update(double deltaTime)
    {
        _startButton.Update(deltaTime);
        _settingsButton.Update(deltaTime);
        _exitButton.Update(deltaTime);
    }

    public void Draw(RenderContext context)
    {
        context.Clear(20, 20, 40);

        var textRenderer = ServiceLocator.Instance.Get<TextRenderer>();
        textRenderer?.DrawText("title", "LLM Galgame Engine", 440, 200, 255, 255, 255);

        _startButton.Draw(context);
        _settingsButton.Draw(context);
        _exitButton.Draw(context);
    }

    public void HandleInput(SDL_Event evt)
    {
        switch (evt.type)
        {
            case SDL_EventType.SDL_MOUSEMOTION:
                var mousePos = new Vector2(evt.GetMouseX(), evt.GetMouseY());
                _startButton.HandleMouseMove(mousePos);
                _settingsButton.HandleMouseMove(mousePos);
                _exitButton.HandleMouseMove(mousePos);
                break;

            case SDL_EventType.SDL_MOUSEBUTTONDOWN:
                var downPos = new Vector2(evt.GetMouseX(), evt.GetMouseY());
                _startButton.HandleMouseDown(downPos);
                _settingsButton.HandleMouseDown(downPos);
                _exitButton.HandleMouseDown(downPos);
                break;

            case SDL_EventType.SDL_MOUSEBUTTONUP:
                var upPos = new Vector2(evt.GetMouseX(), evt.GetMouseY());
                _startButton.HandleMouseUp(upPos);
                _settingsButton.HandleMouseUp(upPos);
                _exitButton.HandleMouseUp(upPos);
                break;
        }
    }
}
