using System.Numerics;
using LLMAssistant.Renderer.SDL2;
using static LLMAssistant.Renderer.SDL2.SDL2Native;

namespace LLMAssistant.Renderer;

public class Sprite : IDisposable
{
    private IntPtr _texture;
    private bool _disposed;

    public IntPtr Texture => _texture;
    public int Width { get; private set; }
    public int Height { get; private set; }
    public Vector2 Position { get; set; }
    public Vector2 Scale { get; set; } = Vector2.One;
    public float Rotation { get; set; }
    public Vector2 Origin { get; set; }
    public SDL_Color Color { get; set; } = new() { r = 255, g = 255, b = 255, a = 255 };
    public bool Visible { get; set; } = true;

    public Sprite(IntPtr texture, int width, int height)
    {
        _texture = texture;
        Width = width;
        Height = height;
    }

    public static Sprite? FromFile(RenderContext context, string path)
    {
        var surface = IMG_Load(path);
        if (surface == IntPtr.Zero) return null;

        var texture = SDL_CreateTextureFromSurface(context.Renderer, surface);
        SDL_QueryTexture(texture, out _, out _, out var w, out var h);
        SDL_FreeSurface(surface);

        if (texture == IntPtr.Zero) return null;

        return new Sprite(texture, w, h);
    }

    public void Draw(RenderContext context)
    {
        if (!Visible || _texture == IntPtr.Zero) return;

        var dstRect = new SDL_Rect
        {
            x = (int)(Position.X - Origin.X * Scale.X),
            y = (int)(Position.Y - Origin.Y * Scale.Y),
            w = (int)(Width * Scale.X),
            h = (int)(Height * Scale.Y)
        };

        SDL_SetTextureColorMod(_texture, Color.r, Color.g, Color.b);
        SDL_SetTextureAlphaMod(_texture, Color.a);

        if (Math.Abs(Rotation) > 0.001f)
        {
            var center = new SDL_Point
            {
                x = (int)(Origin.X * Scale.X),
                y = (int)(Origin.Y * Scale.Y)
            };
            SDL_RenderCopyEx(context.Renderer, _texture, IntPtr.Zero, ref dstRect,
                Rotation, ref center, 0);
        }
        else
        {
            context.DrawTexture(_texture, dstRect);
        }
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            if (_texture != IntPtr.Zero)
            {
                SDL_DestroyTexture(_texture);
                _texture = IntPtr.Zero;
            }
            _disposed = true;
        }
    }
}
