using System.Collections.Concurrent;

namespace LLMAssistant.Core.Events;

public class EventBus
{
    private static readonly Lazy<EventBus> _instance = new(() => new EventBus());
    public static EventBus Instance => _instance.Value;

    private readonly ConcurrentDictionary<Type, List<Delegate>> _handlers = new();

    public void Subscribe<T>(Action<T> handler) where T : class
    {
        var handlers = _handlers.GetOrAdd(typeof(T), _ => []);
        lock (handlers)
        {
            handlers.Add(handler);
        }
    }

    public void Unsubscribe<T>(Action<T> handler) where T : class
    {
        if (_handlers.TryGetValue(typeof(T), out var handlers))
        {
            lock (handlers)
            {
                handlers.Remove(handler);
            }
        }
    }

    public void Publish<T>(T eventData) where T : class
    {
        if (!_handlers.TryGetValue(typeof(T), out var handlers)) return;

        List<Delegate> snapshot;
        lock (handlers)
        {
            snapshot = [.. handlers];
        }
        foreach (var handler in snapshot)
        {
            if (handler is Action<T> typedHandler)
            {
                typedHandler(eventData);
            }
        }
    }

    public void Clear()
    {
        _handlers.Clear();
    }
}
