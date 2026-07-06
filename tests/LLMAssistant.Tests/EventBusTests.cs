using LLMAssistant.Core.Events;

namespace LLMAssistant.Tests;

public class EventBusTests
{
    [Fact]
    public void Subscribe_And_Publish_InvokesHandler()
    {
        var bus = new EventBus();
        string? received = null;
        bus.Subscribe<TestEvent>(e => received = e.Message);

        bus.Publish(new TestEvent("hello"));

        Assert.Equal("hello", received);
    }

    [Fact]
    public void Unsubscribe_StopsReceiving()
    {
        var bus = new EventBus();
        string? received = null;
        Action<TestEvent> handler = e => received = e.Message;
        bus.Subscribe(handler);

        bus.Unsubscribe(handler);
        bus.Publish(new TestEvent("hello"));

        Assert.Null(received);
    }

    [Fact]
    public void Publish_MultipleSubscribers_AllInvoked()
    {
        var bus = new EventBus();
        int count = 0;
        bus.Subscribe<TestEvent>(_ => count++);
        bus.Subscribe<TestEvent>(_ => count++);

        bus.Publish(new TestEvent("test"));

        Assert.Equal(2, count);
    }

    private record TestEvent(string Message);
}
