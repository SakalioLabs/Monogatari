using System.Collections.Concurrent;

namespace LLMAssistant.Core.Services;

public class ServiceLocator
{
    private static readonly Lazy<ServiceLocator> _instance = new(() => new ServiceLocator());
    public static ServiceLocator Instance => _instance.Value;

    private readonly ConcurrentDictionary<Type, object> _services = new();

    public void Register<T>(T service) where T : class
    {
        _services[typeof(T)] = service;
    }

    public void RegisterAs<TInterface, TService>(TService service)
        where TInterface : class
        where TService : class, TInterface
    {
        _services[typeof(TInterface)] = service;
    }

    public T? Get<T>() where T : class
    {
        return _services.TryGetValue(typeof(T), out var service) ? service as T : null;
    }

    public T GetRequired<T>() where T : class
    {
        return Get<T>() ?? throw new InvalidOperationException($"Service {typeof(T).Name} not registered");
    }

    public bool TryGet<T>(out T? service) where T : class
    {
        if (_services.TryGetValue(typeof(T), out var s))
        {
            service = s as T;
            return service != null;
        }
        service = null;
        return false;
    }

    public void Clear()
    {
        _services.Clear();
    }
}
