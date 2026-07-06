using System.Runtime.InteropServices;
using LLMAssistant.Renderer.SDL2;
using static LLMAssistant.Renderer.SDL2.SDL2Native;

namespace LLMAssistant.Renderer;

public class TextRenderer : IDisposable
{
    private readonly Dictionary<string, IntPtr> _fonts = new();
    private readonly RenderContext _context;
    private bool _disposed;

    public TextRenderer(RenderContext context)
    {
        _context = context;
        if (TTF_Init() != 0)
        {
            throw new Exception($"TTF Init failed: {Marshal.PtrToStringUTF8(SDL_GetError())}");
        }
    }

    public bool LoadFont(string name, string path, int size)
    {
        var font = TTF_OpenFont(path, size);
        if (font == IntPtr.Zero)
        {
            Console.WriteLine($"Failed to load font {path}: {Marshal.PtrToStringUTF8(SDL_GetError())}");
            return false;
        }
        _fonts[name] = font;
        return true;
    }

    public void DrawText(string fontName, string text, int x, int y,
        byte r = 255, byte g = 255, byte b = 255, byte a = 255)
    {
        if (!_fonts.TryGetValue(fontName, out var font)) return;

        var color = new SDL_Color { r = r, g = g, b = b, a = a };
        var surface = TTF_RenderUTF8_Blended(font, text, color);

        if (surface == IntPtr.Zero) return;

        var texture = SDL_CreateTextureFromSurface(_context.Renderer, surface);
        SDL_FreeSurface(surface);

        if (texture == IntPtr.Zero) return;

        SDL_QueryTexture(texture, out _, out _, out var w, out var h);
        var dstRect = new SDL_Rect { x = x, y = y, w = w, h = h };
        _context.DrawTexture(texture, dstRect);
        SDL_DestroyTexture(texture);
    }

    public (int width, int height) MeasureText(string fontName, string text)
    {
        if (!_fonts.TryGetValue(fontName, out var font)) return (0, 0);
        TTF_SizeUTF8(font, text, out var w, out var h);
        return (w, h);
    }

    public void DrawTextWrapped(string fontName, string text, int x, int y, int maxWidth,
        byte r = 255, byte g = 255, byte b = 255, byte a = 255)
    {
        if (!_fonts.TryGetValue(fontName, out var font)) return;

        var color = new SDL_Color { r = r, g = g, b = b, a = a };
        var surface = TTF_RenderUTF8_Blended_Wrapped(font, text, color, (uint)maxWidth);

        if (surface == IntPtr.Zero) return;

        var texture = SDL_CreateTextureFromSurface(_context.Renderer, surface);
        SDL_FreeSurface(surface);

        if (texture == IntPtr.Zero) return;

        SDL_QueryTexture(texture, out _, out _, out var w, out var h);
        var dstRect = new SDL_Rect { x = x, y = y, w = w, h = h };
        _context.DrawTexture(texture, dstRect);
        SDL_DestroyTexture(texture);
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            foreach (var font in _fonts.Values)
            {
                TTF_CloseFont(font);
            }
            _fonts.Clear();
            TTF_Quit();
            _disposed = true;
        }
    }
}
