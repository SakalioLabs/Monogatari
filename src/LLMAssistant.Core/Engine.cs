using LLMAssistant.Core.Services;
using LLMAssistant.Core.Events;
using LLMAssistant.Core.Time;

namespace LLMAssistant.Core;

public class Engine
{
    private bool _running;
    private readonly List<IGameService> _services = [];

    public bool IsRunning => _running;
    public ServiceLocator Services { get; } = ServiceLocator.Instance;
    public EventBus Events { get; } = EventBus.Instance;
    public GameClock Clock { get; } = new();

    public Engine()
    {
        Services.Register(Events);
        Services.Register(Clock);
    }

    public void RegisterService(IGameService service)
    {
        _services.Add(service);
        // Register under the concrete runtime type so Get<T>() works
        var method = typeof(ServiceLocator).GetMethod("Register")!
            .MakeGenericMethod(service.GetType());
        method.Invoke(Services, [service]);
        service.Initialize();
    }

    public void RegisterServiceAs<TInterface>(TInterface service)
        where TInterface : class
    {
        if (service is IGameService gameService)
            _services.Add(gameService);
        Services.Register(service);
        if (service is IGameService initService)
            initService.Initialize();
    }

    public void Initialize()
    {
        Clock.Start();
        _running = true;
    }

    public void Update()
    {
        Clock.Tick();
        foreach (var service in _services)
        {
            service.Update(Clock.DeltaTime);
        }
    }

    public void Shutdown()
    {
        _running = false;
        foreach (var service in _services)
        {
            service.Shutdown();
        }
        _services.Clear();
        Services.Clear();
        Events.Clear();
    }

    public void Stop() => _running = false;
}
