using System.Runtime.InteropServices;

namespace LLMAssistant.Renderer.SDL2;

public static class SDL2Native
{
    private const string SDL2Lib = "SDL2";
    private const string SDL2TtfLib = "SDL2_ttf";
    private const string SDL2ImageLib = "SDL2_image";

    // Init / Quit
    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_Init(uint flags);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_Quit();

    public const uint SDL_INIT_VIDEO = 0x00000020;
    public const uint SDL_INIT_AUDIO = 0x00000010;

    // Window
    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr SDL_CreateWindow(string title, int x, int y, int w, int h, uint flags);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_DestroyWindow(IntPtr window);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_SetWindowSize(IntPtr window, int w, int h);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_SetWindowTitle(IntPtr window, string title);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_GetWindowSize(IntPtr window, out int w, out int h);

    public const uint SDL_WINDOW_SHOWN = 0x00000004;
    public const uint SDL_WINDOW_HIDDEN = 0x00000008;
    public const uint SDL_WINDOW_RESIZABLE = 0x00000020;
    public const int SDL_WINDOWPOS_CENTERED = 0x2FFF0000;

    // Renderer
    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr SDL_CreateRenderer(IntPtr window, int index, uint flags);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_DestroyRenderer(IntPtr renderer);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_SetRenderDrawColor(IntPtr renderer, byte r, byte g, byte b, byte a);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_RenderClear(IntPtr renderer);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_RenderPresent(IntPtr renderer);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_SetRenderDrawBlendMode(IntPtr renderer, int blendMode);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_RenderDrawRect(IntPtr renderer, ref SDL_Rect rect);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_RenderFillRect(IntPtr renderer, ref SDL_Rect rect);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_RenderDrawLine(IntPtr renderer, int x1, int y1, int x2, int y2);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_RenderCopy(IntPtr renderer, IntPtr texture, IntPtr srcRect, ref SDL_Rect dstRect);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "SDL_RenderCopy")]
    public static extern int SDL_RenderCopyRef(IntPtr renderer, IntPtr texture, ref SDL_Rect srcRect, ref SDL_Rect dstRect);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_RenderCopyEx(IntPtr renderer, IntPtr texture, IntPtr srcRect, ref SDL_Rect dstRect, double angle, ref SDL_Point center, int flip);

    public const uint SDL_RENDERER_SOFTWARE = 0x00000001;
    public const uint SDL_RENDERER_ACCELERATED = 0x00000002;
    public const uint SDL_RENDERER_PRESENTVSYNC = 0x00000004;

    // Texture
    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr SDL_CreateTextureFromSurface(IntPtr renderer, IntPtr surface);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_DestroyTexture(IntPtr texture);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_QueryTexture(IntPtr texture, out uint format, out int access, out int w, out int h);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_SetTextureColorMod(IntPtr texture, byte r, byte g, byte b);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_SetTextureAlphaMod(IntPtr texture, byte a);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_SetTextureBlendMode(IntPtr texture, int blendMode);

    // Surface
    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_FreeSurface(IntPtr surface);

    // Events
    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int SDL_PollEvent(out SDL_Event evt);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern uint SDL_GetTicks();

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_Delay(uint ms);

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr SDL_GetError();

    [DllImport(SDL2Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void SDL_ClearError();

    // Blend modes
    public const int SDL_BLENDMODE_BLEND = 0x00000001;

    // TTF
    [DllImport(SDL2TtfLib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int TTF_Init();

    [DllImport(SDL2TtfLib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void TTF_Quit();

    [DllImport(SDL2TtfLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "TTF_OpenFont")]
    public static extern IntPtr TTF_OpenFont(string file, int ptsize);

    [DllImport(SDL2TtfLib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void TTF_CloseFont(IntPtr font);

    [DllImport(SDL2TtfLib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr TTF_RenderUTF8_Blended(IntPtr font, string text, SDL_Color color);

    [DllImport(SDL2TtfLib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr TTF_RenderUTF8_Blended_Wrapped(IntPtr font, string text, SDL_Color color, uint wrapLength);

    [DllImport(SDL2TtfLib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int TTF_SizeUTF8(IntPtr font, string text, out int w, out int h);

    // Image
    [DllImport(SDL2ImageLib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr IMG_Load(string file);

    [DllImport(SDL2ImageLib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int IMG_Init(int flags);

    [DllImport(SDL2ImageLib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void IMG_Quit();

    public const int IMG_INIT_PNG = 0x00000001;
    public const int IMG_INIT_JPG = 0x00000002;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_Rect
{
    public int x, y, w, h;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_Point
{
    public int x, y;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_Color
{
    public byte r, g, b, a;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_Event
{
    public uint type;
    // Union: 56 bytes of raw data (biggest SDL event is ~56 bytes)
    public unsafe fixed byte data[56];

    // Helper accessors - read fields from the raw union bytes
    // All SDL events start with: type(4) + timestamp(4) + windowID(4) = 12 bytes header
    // Then event-specific fields follow

    public int GetKeySym()
    {
        // KeyboardEvent: type(4)+ts(4)+wid(4)+state(1)+repeat(1)+pad(2)+keysym.scancode(4)+keysym.sym(4) = offset 20
        unsafe { fixed (byte* p = data) { return *(int*)(p + 16); } }
    }

    public int GetMouseX()
    {
        // MouseEvent: type(4)+ts(4)+wid(4)+which(4)+state(4)+x(4) = offset 20
        // ButtonEvent: type(4)+ts(4)+wid(4)+which(4)+button(1)+state(1)+clicks(1)+pad(1)+x(4) = offset 20
        unsafe { fixed (byte* p = data) { return *(int*)(p + 16); } }
    }

    public int GetMouseY()
    {
        // x is at offset 16, y is at offset 20 (relative to union start)
        unsafe { fixed (byte* p = data) { return *(int*)(p + 20); } }
    }

    public byte GetWindowEvent()
    {
        // WindowEvent: type(4)+ts(4)+wid(4)+event(1) = offset 12
        unsafe { fixed (byte* p = data) { return *(p + 8); } }
    }

    public int GetWindowData1()
    {
        // WindowEvent: ...+pad(3)+data1(4) = offset 16
        unsafe { fixed (byte* p = data) { return *(int*)(p + 12); } }
    }

    public int GetWindowData2()
    {
        // WindowEvent: ...+data1(4)+data2(4) = offset 20
        unsafe { fixed (byte* p = data) { return *(int*)(p + 16); } }
    }
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_KeyboardEvent
{
    public uint type;
    public uint timestamp;
    public uint windowID;
    public byte state;
    public byte repeat;
    public byte padding2;
    public byte padding3;
    public SDL_Keysym keysym;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_Keysym
{
    public int scancode;
    public int sym;
    public ushort mod;
    public uint unused;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_MouseMotionEvent
{
    public uint type;
    public uint timestamp;
    public uint windowID;
    public uint which;
    public uint state;
    public int x, y;
    public int xrel, yrel;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_MouseButtonEvent
{
    public uint type;
    public uint timestamp;
    public uint windowID;
    public uint which;
    public byte button;
    public byte state;
    public byte clicks;
    public byte padding1;
    public int x, y;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_MouseWheelEvent
{
    public uint type;
    public uint timestamp;
    public uint windowID;
    public uint which;
    public int x, y;
    public uint direction;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_WindowEvent
{
    public uint type;
    public uint timestamp;
    public uint windowID;
    public byte @event;
    public byte padding1;
    public byte padding2;
    public byte padding3;
    public int data1, data2;
}

[StructLayout(LayoutKind.Sequential)]
public struct SDL_TextInputEvent
{
    public uint type;
    public uint timestamp;
    public uint windowID;
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = 32)]
    public byte[] text;
}

public static class SDL_EventType
{
    public const uint SDL_QUIT = 0x100;
    public const uint SDL_WINDOWEVENT = 0x200;
    public const uint SDL_KEYDOWN = 0x300;
    public const uint SDL_KEYUP = 0x301;
    public const uint SDL_TEXTINPUT = 0x303;
    public const uint SDL_MOUSEMOTION = 0x400;
    public const uint SDL_MOUSEBUTTONDOWN = 0x401;
    public const uint SDL_MOUSEBUTTONUP = 0x402;
    public const uint SDL_MOUSEWHEEL = 0x403;
}

public static class SDL_Scancode
{
    public const int SDL_SCANCODE_ESCAPE = 41;
    public const int SDL_SCANCODE_RETURN = 40;
    public const int SDL_SCANCODE_SPACE = 44;
}

public static class SDL_Keycode
{
    public const int SDLK_ESCAPE = 0x1B;
    public const int SDLK_RETURN = 0x0D;
    public const int SDLK_SPACE = 0x20;
    public const int SDLK_LEFT = 0x40000050;
    public const int SDLK_RIGHT = 0x4000004F;
    public const int SDLK_UP = 0x40000052;
    public const int SDLK_DOWN = 0x40000051;
}
