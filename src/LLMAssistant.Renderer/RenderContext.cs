using System.Runtime.InteropServices;
using LLMAssistant.Renderer.SDL2;
using static LLMAssistant.Renderer.SDL2.SDL2Native;

namespace LLMAssistant.Renderer;

public class RenderContext : IDisposable
{
    private IntPtr _renderer;
    private bool _disposed;

    public IntPtr Renderer => _renderer;

    public RenderContext(
        IntPtr window,
        RendererRuntimeMode runtimeMode = RendererRuntimeMode.Interactive)
    {
        var rendererFlags = runtimeMode == RendererRuntimeMode.Headless
            ? SDL_RENDERER_SOFTWARE
            : SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC;
        _renderer = SDL_CreateRenderer(window, -1, rendererFlags);

        if (_renderer == IntPtr.Zero)
        {
            throw new Exception($"Renderer creation failed: {Marshal.PtrToStringUTF8(SDL_GetError())}");
        }

        SDL_SetRenderDrawBlendMode(_renderer, SDL_BLENDMODE_BLEND);
    }

    public void Clear(byte r = 0, byte g = 0, byte b = 0, byte a = 255)
    {
        SDL_SetRenderDrawColor(_renderer, r, g, b, a);
        SDL_RenderClear(_renderer);
    }

    public void Present()
    {
        SDL_RenderPresent(_renderer);
    }

    public void DrawRect(int x, int y, int w, int h, byte r, byte g, byte b, byte a = 255)
    {
        var rect = new SDL_Rect { x = x, y = y, w = w, h = h };
        SDL_SetRenderDrawColor(_renderer, r, g, b, a);
        SDL_RenderDrawRect(_renderer, ref rect);
    }

    public void FillRect(int x, int y, int w, int h, byte r, byte g, byte b, byte a = 255)
    {
        var rect = new SDL_Rect { x = x, y = y, w = w, h = h };
        SDL_SetRenderDrawColor(_renderer, r, g, b, a);
        SDL_RenderFillRect(_renderer, ref rect);
    }

    public void DrawLine(int x1, int y1, int x2, int y2, byte r, byte g, byte b, byte a = 255)
    {
        SDL_SetRenderDrawColor(_renderer, r, g, b, a);
        SDL_RenderDrawLine(_renderer, x1, y1, x2, y2);
    }

    public void DrawTexture(IntPtr texture, SDL_Rect dstRect)
    {
        SDL_RenderCopy(_renderer, texture, IntPtr.Zero, ref dstRect);
    }

    public void DrawTexture(IntPtr texture, SDL_Rect srcRect, SDL_Rect dstRect)
    {
        SDL_RenderCopyRef(_renderer, texture, ref srcRect, ref dstRect);
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            if (_renderer != IntPtr.Zero)
            {
                SDL_DestroyRenderer(_renderer);
                _renderer = IntPtr.Zero;
            }
            _disposed = true;
        }
    }
}
