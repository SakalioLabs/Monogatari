using System.Net;
using System.Reflection;
using LLMAssistant.AI.API;

namespace LLMAssistant.Tests;

public class APIEngineTests
{
    [Fact]
    public void RedactSensitiveText_RemovesTokenLikeValuesAndSecretAssignments()
    {
        var openAiKey = "sk-" + new string('A', 28);
        var githubKey = "github_pat_" + new string('B', 32);
        var text = "provider failed " +
            $"{{\"api_key\":\"plain-secret-value\",\"authorization\":\"Bearer {openAiKey}\"}} " +
            $"https://example.test/v1?access_token={githubKey}&token=session-secret " +
            "x-api-key: custom-header-secret";

        var redacted = APIEngine.RedactSensitiveText(text);

        Assert.DoesNotContain(openAiKey, redacted);
        Assert.DoesNotContain(githubKey, redacted);
        Assert.DoesNotContain("plain-secret-value", redacted);
        Assert.DoesNotContain("session-secret", redacted);
        Assert.DoesNotContain("custom-header-secret", redacted);
        Assert.Contains("\"api_key\":\"<redacted>\"", redacted);
        Assert.Contains("access_token=<redacted>", redacted);
        Assert.Contains("token=<redacted>", redacted);
    }

    [Fact]
    public async Task InferAsync_RedactsSensitiveProviderErrorBodies()
    {
        var openAiKey = "sk-" + new string('C', 28);
        var plainSecret = "plain-provider-secret";
        var engine = NewEngineWithClient(new StaticResponseHandler(_ =>
            Task.FromResult(new HttpResponseMessage(HttpStatusCode.Unauthorized)
            {
                Content = new StringContent(
                    $"{{\"error\":\"bad key\",\"api_key\":\"{plainSecret}\",\"authorization\":\"Bearer {openAiKey}\"}}")
            })));

        var result = await engine.InferAsync("hello");

        Assert.False(result.Success);
        Assert.NotNull(result.Error);
        Assert.Contains("<redacted>", result.Error);
        Assert.DoesNotContain(openAiKey, result.Error);
        Assert.DoesNotContain(plainSecret, result.Error);
    }

    [Fact]
    public async Task InferAsync_RedactsSensitiveRequestExceptions()
    {
        var githubKey = "github_pat_" + new string('D', 32);
        var engine = NewEngineWithClient(new StaticResponseHandler(_ =>
            throw new HttpRequestException($"network failed for https://example.test?access_token={githubKey}")));

        var result = await engine.InferAsync("hello");

        Assert.False(result.Success);
        Assert.NotNull(result.Error);
        Assert.Contains("access_token=<redacted>", result.Error);
        Assert.DoesNotContain(githubKey, result.Error);
    }

    private static APIEngine NewEngineWithClient(HttpMessageHandler handler)
    {
        var engine = new APIEngine(new APIConfig
        {
            BaseUrl = "https://example.test/v1",
            Model = "test-model"
        });
        var client = new HttpClient(handler)
        {
            BaseAddress = new Uri("https://example.test")
        };
        var field = typeof(APIEngine).GetField("_httpClient", BindingFlags.Instance | BindingFlags.NonPublic);
        Assert.NotNull(field);
        field.SetValue(engine, client);
        return engine;
    }

    private sealed class StaticResponseHandler : HttpMessageHandler
    {
        private readonly Func<HttpRequestMessage, Task<HttpResponseMessage>> _send;

        public StaticResponseHandler(Func<HttpRequestMessage, Task<HttpResponseMessage>> send)
        {
            _send = send;
        }

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            return _send(request);
        }
    }
}
