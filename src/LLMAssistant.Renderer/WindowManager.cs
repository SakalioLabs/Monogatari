using System.Runtime.InteropServices;
using LLMAssistant.Renderer.SDL2;
using static LLMAssistant.Renderer.SDL2.SDL2Native;

namespace LLMAssistant.Renderer;

public class WindowManager : IDisposable
{
    private IntPtr _window;
    private bool _disposed;
    private readonly RendererRuntimeMode _runtimeMode;

    public IntPtr Window => _window;
    public int Width { get; private set; }
    public int Height { get; private set; }
    public string Title { get; private set; }

    public WindowManager(
        string title,
        int width = 1280,
        int height = 720,
        RendererRuntimeMode runtimeMode = RendererRuntimeMode.Interactive)
    {
        Title = title;
        Width = width;
        Height = height;
        _runtimeMode = runtimeMode;
    }

    public bool Initialize()
    {
        var initFlags = _runtimeMode == RendererRuntimeMode.Headless
            ? SDL_INIT_VIDEO
            : SDL_INIT_VIDEO | SDL_INIT_AUDIO;
        if (SDL_Init(initFlags) != 0)
        {
            Console.WriteLine($"SDL Init Error: {Marshal.PtrToStringUTF8(SDL_GetError())}");
            return false;
        }

        _window = SDL_CreateWindow(
            Title,
            SDL_WINDOWPOS_CENTERED,
            SDL_WINDOWPOS_CENTERED,
            Width,
            Height,
            _runtimeMode == RendererRuntimeMode.Headless
                ? SDL_WINDOW_HIDDEN
                : SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE
        );

        if (_window == IntPtr.Zero)
        {
            Console.WriteLine($"Window Creation Error: {Marshal.PtrToStringUTF8(SDL_GetError())}");
            return false;
        }

        return true;
    }

    public void SetSize(int width, int height)
    {
        Width = width;
        Height = height;
        SDL_SetWindowSize(_window, width, height);
    }

    public void SetTitle(string title)
    {
        Title = title;
        SDL_SetWindowTitle(_window, title);
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            if (_window != IntPtr.Zero)
            {
                SDL_DestroyWindow(_window);
                _window = IntPtr.Zero;
            }
            SDL_Quit();
            _disposed = true;
        }
    }
}
