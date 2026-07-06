using System.Numerics;

namespace LLMAssistant.Renderer.UI;

public class ChoicePanel : UIElement
{
    private readonly List<Button> _buttons = [];
    private int _selectedIndex = -1;

    public int Spacing { get; set; } = 8;
    public int ButtonHeight { get; set; } = 48;
    public int MaxVisibleChoices { get; set; } = 6;

    public IReadOnlyList<Button> Buttons => _buttons;
    public int SelectedIndex => _selectedIndex;

    public event Action<int, string>? OnChoiceSelected;

    public void SetChoices(IReadOnlyList<string> choices)
    {
        _buttons.Clear();
        Children.Clear();
        _selectedIndex = -1;

        for (int i = 0; i < Math.Min(choices.Count, MaxVisibleChoices); i++)
        {
            var button = new Button
            {
                Text = choices[i],
                Size = new Vector2(Size.X, ButtonHeight),
                Position = new Vector2(0, i * (ButtonHeight + Spacing))
            };

            var index = i;
            button.OnClick += () =>
            {
                _selectedIndex = index;
                OnChoiceSelected?.Invoke(index, choices[index]);
            };

            _buttons.Add(button);
            AddChild(button);
        }

        Size = new Vector2(Size.X, _buttons.Count * (ButtonHeight + Spacing) - Spacing);
    }

    public void HandleMouseMove(Vector2 mousePos)
    {
        foreach (var button in _buttons)
        {
            button.HandleMouseMove(mousePos);
        }
    }

    public bool HandleMouseDown(Vector2 mousePos)
    {
        foreach (var button in _buttons)
        {
            if (button.HandleMouseDown(mousePos)) return true;
        }
        return false;
    }

    public bool HandleMouseUp(Vector2 mousePos)
    {
        foreach (var button in _buttons)
        {
            if (button.HandleMouseUp(mousePos)) return true;
        }
        return false;
    }

    public override void Update(double deltaTime)
    {
        foreach (var button in _buttons)
        {
            button.Update(deltaTime);
        }
    }

    public override void Draw(RenderContext context)
    {
        if (!Visible) return;
        foreach (var button in _buttons)
        {
            button.Draw(context);
        }
    }
}
