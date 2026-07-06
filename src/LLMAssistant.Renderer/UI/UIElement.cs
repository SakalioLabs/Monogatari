using System.Numerics;

namespace LLMAssistant.Renderer.UI;

public abstract class UIElement
{
    public Vector2 Position { get; set; }
    public Vector2 Size { get; set; }
    public bool Visible { get; set; } = true;
    public bool Enabled { get; set; } = true;
    public float Alpha { get; set; } = 1.0f;
    public UIElement? Parent { get; set; }
    public List<UIElement> Children { get; } = [];

    public Vector2 AbsolutePosition
    {
        get
        {
            var pos = Position;
            var parent = Parent;
            while (parent != null)
            {
                pos += parent.Position;
                parent = parent.Parent;
            }
            return pos;
        }
    }

    public bool ContainsPoint(Vector2 point)
    {
        var absPos = AbsolutePosition;
        return point.X >= absPos.X && point.X <= absPos.X + Size.X &&
               point.Y >= absPos.Y && point.Y <= absPos.Y + Size.Y;
    }

    public virtual void AddChild(UIElement child)
    {
        child.Parent = this;
        Children.Add(child);
    }

    public virtual void RemoveChild(UIElement child)
    {
        child.Parent = null;
        Children.Remove(child);
    }

    public abstract void Update(double deltaTime);
    public abstract void Draw(RenderContext context);

    public virtual void DrawChildren(RenderContext context)
    {
        foreach (var child in Children)
        {
            if (child.Visible)
            {
                child.Draw(context);
                child.DrawChildren(context);
            }
        }
    }
}
