using LLMAssistant.Core.Services;
using LLMAssistant.Core.Events;
using LLMAssistant.Renderer;
using LLMAssistant.Renderer.SDL2;

namespace LLMAssistant.Game.Scenes;

public class SceneManager : IGameService
{
    private readonly Stack<IScene> _scenes = new();
    private readonly Dictionary<string, Func<IScene>> _sceneFactories = new();

    public string ServiceName => "SceneManager";
    public IScene? CurrentScene => _scenes.Count > 0 ? _scenes.Peek() : null;
    public int SceneCount => _scenes.Count;

    public void RegisterScene(string name, Func<IScene> factory)
    {
        _sceneFactories[name] = factory;
    }

    public void PushScene(string name)
    {
        if (!_sceneFactories.TryGetValue(name, out var factory))
            throw new ArgumentException($"Scene '{name}' not registered");

        CurrentScene?.Pause();
        var scene = factory();
        _scenes.Push(scene);
        scene.Enter();
        EventBus.Instance.Publish(new SceneChangedEvent(name));
    }

    public void PopScene()
    {
        if (_scenes.Count == 0) return;

        var scene = _scenes.Pop();
        scene.Exit();
        CurrentScene?.Resume();
    }

    public void ReplaceScene(string name)
    {
        if (_scenes.Count > 0)
        {
            var old = _scenes.Pop();
            old.Exit();
        }
        PushScene(name);
    }

    public void Initialize() { }

    public void Update(double deltaTime)
    {
        CurrentScene?.Update(deltaTime);
    }

    public void Draw(RenderContext context)
    {
        var scenesList = _scenes.Reverse().ToList();
        foreach (var scene in scenesList)
        {
            scene.Draw(context);
            if (scene.IsOpaque) break;
        }
    }

    public void HandleInput(SDL_Event evt)
    {
        CurrentScene?.HandleInput(evt);
    }

    public void Shutdown()
    {
        while (_scenes.Count > 0)
        {
            _scenes.Pop().Exit();
        }
    }
}
