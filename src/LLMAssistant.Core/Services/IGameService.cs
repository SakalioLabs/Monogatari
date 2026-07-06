namespace LLMAssistant.Core.Services;

public interface IGameService
{
    string ServiceName { get; }
    void Initialize();
    void Update(double deltaTime);
    void Shutdown();
}
