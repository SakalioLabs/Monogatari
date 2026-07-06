using LLMAssistant.Core.Services;

namespace LLMAssistant.Tests;

public class ServiceLocatorTests
{
    [Fact]
    public void Register_And_Get_ReturnsService()
    {
        var locator = new ServiceLocator();
        var service = new TestService();
        locator.Register(service);

        var result = locator.Get<TestService>();

        Assert.Same(service, result);
    }

    [Fact]
    public void Get_Unregistered_ReturnsNull()
    {
        var locator = new ServiceLocator();

        var result = locator.Get<TestService>();

        Assert.Null(result);
    }

    [Fact]
    public void GetRequired_Unregistered_Throws()
    {
        var locator = new ServiceLocator();

        Assert.Throws<InvalidOperationException>(() => locator.GetRequired<TestService>());
    }

    [Fact]
    public void Clear_RemovesAllServices()
    {
        var locator = new ServiceLocator();
        locator.Register(new TestService());

        locator.Clear();

        Assert.Null(locator.Get<TestService>());
    }

    private class TestService { }
}
