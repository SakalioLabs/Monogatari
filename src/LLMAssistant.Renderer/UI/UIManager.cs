using System.Numerics;
using LLMAssistant.Core.Services;

namespace LLMAssistant.Renderer.UI;

public class UIManager : IGameService
{
    private readonly List<UIElement> _rootElements = [];
    private readonly RenderContext _renderContext;

    public string ServiceName => "UIManager";
    public IReadOnlyList<UIElement> RootElements => _rootElements;

    public UIManager(RenderContext renderContext)
    {
        _renderContext = renderContext;
    }

    public void AddElement(UIElement element)
    {
        _rootElements.Add(element);
    }

    public void RemoveElement(UIElement element)
    {
        _rootElements.Remove(element);
    }

    public void Clear()
    {
        _rootElements.Clear();
    }

    public void HandleMouseMove(int x, int y)
    {
        var pos = new Vector2(x, y);
        foreach (var element in _rootElements)
        {
            if (element is Button btn) btn.HandleMouseMove(pos);
            if (element is ChoicePanel panel) panel.HandleMouseMove(pos);
        }
    }

    public void HandleMouseDown(int x, int y)
    {
        var pos = new Vector2(x, y);
        foreach (var element in _rootElements)
        {
            if (element is Button btn) btn.HandleMouseDown(pos);
            if (element is ChoicePanel panel) panel.HandleMouseDown(pos);
        }
    }

    public void HandleMouseUp(int x, int y)
    {
        var pos = new Vector2(x, y);
        foreach (var element in _rootElements)
        {
            if (element is Button btn) btn.HandleMouseUp(pos);
            if (element is ChoicePanel panel) panel.HandleMouseUp(pos);
        }
    }

    public void Initialize() { }

    public void Update(double deltaTime)
    {
        foreach (var element in _rootElements)
        {
            if (element.Visible)
            {
                element.Update(deltaTime);
            }
        }
    }

    public void Draw()
    {
        foreach (var element in _rootElements)
        {
            if (element.Visible)
            {
                element.Draw(_renderContext);
                element.DrawChildren(_renderContext);
            }
        }
    }

    public void Shutdown()
    {
        _rootElements.Clear();
    }
}
