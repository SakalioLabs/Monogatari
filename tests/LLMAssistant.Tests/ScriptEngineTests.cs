using LLMAssistant.Scripting;

namespace LLMAssistant.Tests;

public class ScriptEngineTests
{
    [Fact]
    public void SetVariable_And_GetVariable()
    {
        var engine = new ScriptEngine();
        engine.SetVariable("name", "Sakura");

        Assert.Equal("Sakura", engine.GetVariable("name"));
    }

    [Fact]
    public void Execute_SetFlag()
    {
        var engine = new ScriptEngine();
        engine.Execute("setFlag('met_sakura', true)");

        Assert.Equal("true", engine.GetVariable("met_sakura"));
    }

    [Fact]
    public void EvaluateCondition_HasFlag_ReturnsTrue()
    {
        var engine = new ScriptEngine();
        engine.SetVariable("met_sakura", "true");

        Assert.True(engine.EvaluateCondition("hasFlag('met_sakura')"));
    }

    [Fact]
    public void EvaluateCondition_HasFlag_ReturnsFalse()
    {
        var engine = new ScriptEngine();

        Assert.False(engine.EvaluateCondition("hasFlag('met_sakura')"));
    }

    [Fact]
    public void RegisterFunction_And_Call()
    {
        var engine = new ScriptEngine();
        var called = false;
        engine.RegisterFunction("testFunc", args =>
        {
            called = true;
            return args.Length > 0 ? args[0] : null;
        });

        engine.Execute("testFunc('hello')");
        Assert.True(called);
    }
}
