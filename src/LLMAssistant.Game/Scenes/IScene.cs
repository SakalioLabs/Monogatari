using LLMAssistant.Renderer;
using LLMAssistant.Renderer.SDL2;

namespace LLMAssistant.Game.Scenes;

public interface IScene
{
    string Name { get; }
    bool IsOpaque { get; set; }
    void Enter();
    void Exit();
    void Pause();
    void Resume();
    void Update(double deltaTime);
    void Draw(RenderContext context);
    void HandleInput(SDL_Event evt);
}
