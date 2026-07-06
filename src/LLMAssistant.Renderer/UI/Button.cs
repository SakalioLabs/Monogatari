using System.Numerics;
using LLMAssistant.Core.Services;

namespace LLMAssistant.Renderer.UI;

public class Button : UIElement
{
    public string Text { get; set; } = "";
    public string FontName { get; set; } = "default";
    public byte R { get; set; } = 60;
    public byte G { get; set; } = 60;
    public byte B { get; set; } = 80;
    public byte HoverR { get; set; } = 80;
    public byte HoverG { get; set; } = 80;
    public byte HoverB { get; set; } = 120;
    public byte TextR { get; set; } = 255;
    public byte TextG { get; set; } = 255;
    public byte TextB { get; set; } = 255;
    public bool IsHovered { get; private set; }
    public bool IsPressed { get; private set; }

    public event Action? OnClick;

    public void HandleMouseMove(Vector2 mousePos)
    {
        IsHovered = ContainsPoint(mousePos);
    }

    public bool HandleMouseDown(Vector2 mousePos)
    {
        if (ContainsPoint(mousePos))
        {
            IsPressed = true;
            return true;
        }
        return false;
    }

    public bool HandleMouseUp(Vector2 mousePos)
    {
        if (IsPressed && ContainsPoint(mousePos))
        {
            IsPressed = false;
            OnClick?.Invoke();
            return true;
        }
        IsPressed = false;
        return false;
    }

    public override void Update(double deltaTime) { }

    public override void Draw(RenderContext context)
    {
        if (!Visible) return;

        var absPos = AbsolutePosition;
        var r = IsHovered ? HoverR : R;
        var g = IsHovered ? HoverG : G;
        var b = IsHovered ? HoverB : B;

        context.FillRect((int)absPos.X, (int)absPos.Y, (int)Size.X, (int)Size.Y,
            r, g, b, (byte)(255 * Alpha));

        context.DrawRect((int)absPos.X, (int)absPos.Y, (int)Size.X, (int)Size.Y,
            150, 150, 180, (byte)(255 * Alpha));

        var textRenderer = ServiceLocator.Instance.Get<TextRenderer>();
        if (textRenderer != null)
        {
            var (tw, th) = textRenderer.MeasureText(FontName, Text);
            var tx = absPos.X + (Size.X - tw) / 2;
            var ty = absPos.Y + (Size.Y - th) / 2;
            textRenderer.DrawText(FontName, Text, (int)tx, (int)ty,
                TextR, TextG, TextB, (byte)(255 * Alpha));
        }
    }
}
