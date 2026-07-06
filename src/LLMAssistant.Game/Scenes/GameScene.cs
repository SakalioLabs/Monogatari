using System.Numerics;
using LLMAssistant.Core.Services;
using LLMAssistant.Renderer;
using LLMAssistant.Renderer.UI;
using LLMAssistant.Renderer.SDL2;
using LLMAssistant.Game.Dialogue;
using static LLMAssistant.Renderer.SDL2.SDL2Native;

namespace LLMAssistant.Game.Scenes;

public class GameScene : IScene
{
    public string Name => "Game";
    public bool IsOpaque { get; set; } = true;

    private TextBox _dialogueBox = null!;
    private ChoicePanel _choicePanel = null!;
    private TextBox _nameBox = null!;
    private SceneManager _sceneManager = null!;

    public void Enter()
    {
        _sceneManager = ServiceLocator.Instance.GetRequired<SceneManager>();

        _nameBox = new TextBox
        {
            Position = new Vector2(50, 480),
            Size = new Vector2(200, 40),
            BackgroundA = 180,
            TypewriterEnabled = false
        };

        _dialogueBox = new TextBox
        {
            Position = new Vector2(50, 530),
            Size = new Vector2(1180, 150),
            BackgroundA = 200,
            TypewriterEnabled = true,
            TypewriterSpeed = 40
        };

        _choicePanel = new ChoicePanel
        {
            Position = new Vector2(440, 250),
            Size = new Vector2(400, 300),
            Visible = false
        };
        _choicePanel.OnChoiceSelected += OnChoiceSelected;

        // Subscribe to dialogue events
        var dialogueManager = ServiceLocator.Instance.Get<DialogueManager>();
        if (dialogueManager != null)
        {
            dialogueManager.OnShowDialogue += ShowDialogue;
            dialogueManager.OnShowChoices += ShowChoices;
            dialogueManager.OnDialogueEnd += OnDialogueEnd;
        }
    }

    public void Exit()
    {
        var dialogueManager = ServiceLocator.Instance.Get<DialogueManager>();
        if (dialogueManager != null)
        {
            dialogueManager.OnShowDialogue -= ShowDialogue;
            dialogueManager.OnShowChoices -= ShowChoices;
            dialogueManager.OnDialogueEnd -= OnDialogueEnd;
        }
    }

    public void Pause() { }
    public void Resume() { }

    private void OnChoiceSelected(int index, string text)
    {
        _choicePanel.Visible = false;
        var dialogueManager = ServiceLocator.Instance.Get<DialogueManager>();
        dialogueManager?.SelectChoice(index);
    }

    private void OnDialogueEnd()
    {
        _nameBox.Text = "";
        _dialogueBox.Text = "";
    }

    public void ShowDialogue(string characterName, string text)
    {
        _nameBox.Text = characterName;
        _dialogueBox.Text = text;
    }

    public void ShowChoices(IReadOnlyList<string> choices)
    {
        _choicePanel.SetChoices(choices);
        _choicePanel.Visible = true;
    }

    public void Update(double deltaTime)
    {
        _nameBox.Update(deltaTime);
        _dialogueBox.Update(deltaTime);
        _choicePanel.Update(deltaTime);
    }

    public void Draw(RenderContext context)
    {
        context.Clear(30, 30, 50);

        // Draw background placeholder
        context.FillRect(0, 0, 1280, 480, 40, 40, 60);

        _nameBox.Draw(context);
        _dialogueBox.Draw(context);
        _choicePanel.Draw(context);
    }

    public void HandleInput(SDL_Event evt)
    {
        switch (evt.type)
        {
            case SDL_EventType.SDL_KEYDOWN:
                var key = evt.GetKeySym();
                if (key == SDL_Keycode.SDLK_SPACE || key == SDL_Keycode.SDLK_RETURN)
                {
                    if (_dialogueBox.IsTyping)
                        _dialogueBox.SkipTypewriter();
                    else
                    {
                        var dm = ServiceLocator.Instance.Get<DialogueManager>();
                        dm?.Advance();
                    }
                }
                if (key == SDL_Keycode.SDLK_ESCAPE)
                    _sceneManager.PopScene();
                break;

            case SDL_EventType.SDL_MOUSEMOTION:
                _choicePanel.HandleMouseMove(new Vector2(evt.GetMouseX(), evt.GetMouseY()));
                break;

            case SDL_EventType.SDL_MOUSEBUTTONDOWN:
                _choicePanel.HandleMouseDown(new Vector2(evt.GetMouseX(), evt.GetMouseY()));
                break;

            case SDL_EventType.SDL_MOUSEBUTTONUP:
                _choicePanel.HandleMouseUp(new Vector2(evt.GetMouseX(), evt.GetMouseY()));
                break;
        }
    }
}
