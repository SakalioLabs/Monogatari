using System.Numerics;
using LLMAssistant.Core.Services;
using LLMAssistant.Renderer;
using LLMAssistant.Renderer.UI;
using LLMAssistant.Renderer.SDL2;
using static LLMAssistant.Renderer.SDL2.SDL2Native;

namespace LLMAssistant.Game.Scenes;

public class SettingsScene : IScene
{
    public string Name => "Settings";
    public bool IsOpaque { get; set; } = true;

    private Button _backButton = null!;

    public void Enter()
    {
        _backButton = new Button
        {
            Text = "Back",
            Size = new Vector2(200, 50),
            Position = new Vector2(540, 600)
        };
        _backButton.OnClick += () =>
        {
            var sm = ServiceLocator.Instance.GetRequired<SceneManager>();
            sm.PopScene();
        };
    }

    public void Exit() { }
    public void Pause() { }
    public void Resume() { }

    public void Update(double deltaTime) => _backButton.Update(deltaTime);

    public void Draw(RenderContext context)
    {
        context.Clear(30, 30, 50);
        var textRenderer = ServiceLocator.Instance.Get<TextRenderer>();
        textRenderer?.DrawText("title", "Settings", 580, 100, 255, 255, 255);
        _backButton.Draw(context);
    }

    public void HandleInput(SDL_Event evt)
    {
        if (evt.type == SDL_EventType.SDL_MOUSEMOTION)
            _backButton.HandleMouseMove(new Vector2(evt.GetMouseX(), evt.GetMouseY()));
        if (evt.type == SDL_EventType.SDL_MOUSEBUTTONDOWN)
            _backButton.HandleMouseDown(new Vector2(evt.GetMouseX(), evt.GetMouseY()));
        if (evt.type == SDL_EventType.SDL_MOUSEBUTTONUP)
            _backButton.HandleMouseUp(new Vector2(evt.GetMouseX(), evt.GetMouseY()));
    }
}
