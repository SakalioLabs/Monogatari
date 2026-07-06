using System.Numerics;
using LLMAssistant.Core.Services;

namespace LLMAssistant.Renderer.UI;

public class TextBox : UIElement
{
    private string _fullText = "";
    private string _displayedText = "";
    private double _charTimer;
    private int _charIndex;
    private bool _isTyping;

    public string Text
    {
        get => _fullText;
        set
        {
            _fullText = value;
            if (TypewriterEnabled)
            {
                _displayedText = "";
                _charIndex = 0;
                _charTimer = 0;
                _isTyping = true;
            }
            else
            {
                _displayedText = value;
                _isTyping = false;
            }
        }
    }

    public string FontName { get; set; } = "default";
    public byte TextR { get; set; } = 255;
    public byte TextG { get; set; } = 255;
    public byte TextB { get; set; } = 255;
    public byte TextA { get; set; } = 255;
    public byte BackgroundR { get; set; } = 0;
    public byte BackgroundG { get; set; } = 0;
    public byte BackgroundB { get; set; } = 0;
    public byte BackgroundA { get; set; } = 200;
    public int Padding { get; set; } = 16;
    public bool TypewriterEnabled { get; set; } = true;
    public double TypewriterSpeed { get; set; } = 30.0;
    public bool IsTyping => _isTyping;
    public string DisplayedText => _displayedText;

    public event Action? OnTypingComplete;

    public void SkipTypewriter()
    {
        _displayedText = _fullText;
        _isTyping = false;
        OnTypingComplete?.Invoke();
    }

    public override void Update(double deltaTime)
    {
        if (!_isTyping) return;

        _charTimer += deltaTime;
        var charsToAdd = (int)(_charTimer * TypewriterSpeed);
        if (charsToAdd > 0)
        {
            _charTimer = 0;
            var remaining = _fullText.Length - _charIndex;
            var toAdd = Math.Min(charsToAdd, remaining);
            _charIndex += toAdd;
            _displayedText = _fullText[.._charIndex];

            if (_charIndex >= _fullText.Length)
            {
                _isTyping = false;
                OnTypingComplete?.Invoke();
            }
        }
    }

    public override void Draw(RenderContext context)
    {
        if (!Visible) return;

        var absPos = AbsolutePosition;

        context.FillRect(
            (int)absPos.X, (int)absPos.Y,
            (int)Size.X, (int)Size.Y,
            BackgroundR, BackgroundG, BackgroundB, (byte)(BackgroundA * Alpha));

        context.DrawRect(
            (int)absPos.X, (int)absPos.Y,
            (int)Size.X, (int)Size.Y,
            200, 200, 200, (byte)(255 * Alpha));

        var textRenderer = ServiceLocator.Instance.Get<TextRenderer>();
        textRenderer?.DrawTextWrapped(FontName, _displayedText,
            (int)absPos.X + Padding, (int)absPos.Y + Padding,
            (int)Size.X - Padding * 2,
            TextR, TextG, TextB, (byte)(TextA * Alpha));
    }
}
