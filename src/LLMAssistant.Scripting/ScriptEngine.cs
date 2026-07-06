using System.Text;
using LLMAssistant.Core.Services;

namespace LLMAssistant.Scripting;

public class ScriptEngine : IGameService
{
    private readonly Dictionary<string, Func<object?[], object?>> _functions = new();
    private readonly Dictionary<string, object> _variables = new();

    public string ServiceName => "ScriptEngine";

    public void RegisterFunction(string name, Func<object?[], object?> func)
    {
        _functions[name] = func;
    }

    public void SetVariable(string name, object value)
    {
        _variables[name] = value;
    }

    public object? GetVariable(string name)
    {
        return _variables.TryGetValue(name, out var value) ? value : null;
    }

    public T? GetVariable<T>(string name)
    {
        var value = GetVariable(name);
        if (value is T typed) return typed;
        return default;
    }

    public object? Execute(string script)
    {
        if (string.IsNullOrWhiteSpace(script)) return null;

        try
        {
            var lines = script.Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
            object? lastResult = null;

            foreach (var line in lines)
            {
                lastResult = ExecuteLine(line.Trim());
            }

            return lastResult;
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Script execution error: {ex.Message}");
            return null;
        }
    }

    private object? ExecuteLine(string line)
    {
        // Handle setVariable('name', value)
        if (line.StartsWith("setVariable(") || line.StartsWith("setFlag("))
        {
            var args = ParseFunctionArgs(line);
            if (args.Length >= 2)
            {
                var name = args[0].Trim('\'', '"');
                var value = args[1].Trim('\'', '"');
                SetVariable(name, value);
                return value;
            }
        }

        // Handle getVariable('name')
        if (line.StartsWith("getVariable("))
        {
            var args = ParseFunctionArgs(line);
            if (args.Length >= 1)
            {
                var name = args[0].Trim('\'', '"');
                return GetVariable(name);
            }
        }

        // Handle registered function calls
        var parenIndex = line.IndexOf('(');
        if (parenIndex > 0)
        {
            var funcName = line[..parenIndex].Trim();
            if (_functions.TryGetValue(funcName, out var func))
            {
                var args = ParseFunctionArgs(line);
                return func(args.Cast<object?>().ToArray());
            }
        }

        return null;
    }

    private string[] ParseFunctionArgs(string line)
    {
        var start = line.IndexOf('(');
        var end = line.LastIndexOf(')');
        if (start < 0 || end < 0 || end <= start) return [];

        var argsStr = line.Substring(start + 1, end - start - 1);
        var args = new List<string>();
        var current = new StringBuilder();
        bool inString = false;
        char stringChar = '\0';

        foreach (var c in argsStr)
        {
            if (!inString && (c == '\'' || c == '"'))
            {
                inString = true;
                stringChar = c;
                current.Append(c);
            }
            else if (inString && c == stringChar)
            {
                inString = false;
                current.Append(c);
            }
            else if (!inString && c == ',')
            {
                args.Add(current.ToString().Trim());
                current.Clear();
            }
            else
            {
                current.Append(c);
            }
        }

        if (current.Length > 0)
        {
            args.Add(current.ToString().Trim());
        }

        return args.ToArray();
    }

    public bool EvaluateCondition(string? condition)
    {
        if (string.IsNullOrWhiteSpace(condition)) return true;

        // Handle flag checks: hasFlag('name')
        if (condition.StartsWith("hasFlag("))
        {
            var args = ParseFunctionArgs(condition);
            if (args.Length >= 1)
            {
                var name = args[0].Trim('\'', '"');
                return GetVariable(name) is "true" or true;
            }
        }

        // Handle simple comparisons: variable == value
        if (condition.Contains("=="))
        {
            var parts = condition.Split("==", 2);
            var varName = parts[0].Trim();
            var expected = parts[1].Trim().Trim('\'', '"');
            var actual = GetVariable(varName)?.ToString();
            return actual == expected;
        }

        return true;
    }

    public void Initialize() { }
    public void Update(double deltaTime) { }
    public void Shutdown() { }
}
