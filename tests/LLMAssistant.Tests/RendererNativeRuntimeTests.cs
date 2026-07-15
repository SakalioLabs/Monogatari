using System.Runtime.InteropServices;
using LLMAssistant.Renderer;
using static LLMAssistant.Renderer.SDL2.SDL2Native;

namespace LLMAssistant.Tests;

public class RendererNativeRuntimeTests
{
    [Fact]
    public void WindowsRuntime_LoadsRequiredLibrariesAndExports()
    {
        if (!OperatingSystem.IsWindows())
            return;

        var loaded = new List<nint>();
        try
        {
            loaded.Add(LoadWithExport("SDL2.dll", "SDL_Init"));
            loaded.Add(LoadWithExport("SDL2_image.dll", "IMG_Init"));
            loaded.Add(LoadWithExport("SDL2_ttf.dll", "TTF_Init"));

            foreach (var dependency in new[]
            {
                "libavif-16.dll",
                "libtiff-5.dll",
                "libwebp-7.dll",
                "libwebpdemux-2.dll"
            })
            {
                Assert.True(File.Exists(Path.Combine(AppContext.BaseDirectory, dependency)),
                    $"Missing renderer dependency: {dependency}");
            }

            var licenseDirectory = Path.Combine(AppContext.BaseDirectory, "licenses");
            foreach (var license in new[]
            {
                "SDL2-README.txt",
                "SDL2_image-LICENSE.avif.txt",
                "SDL2_image-LICENSE.dav1d.txt",
                "SDL2_image-LICENSE.tiff.txt",
                "SDL2_image-LICENSE.webp.txt",
                "SDL2_image-README.txt",
                "SDL2_ttf-README.txt"
            })
            {
                Assert.True(File.Exists(Path.Combine(licenseDirectory, license)),
                    $"Missing renderer license: {license}");
            }
        }
        finally
        {
            foreach (var handle in loaded.AsEnumerable().Reverse())
                NativeLibrary.Free(handle);
        }
    }

    [Fact]
    public void WindowsRuntime_RendersHeadlessFramesThroughProductContext()
    {
        if (!OperatingSystem.IsWindows())
            return;

        var previousVideoDriver = Environment.GetEnvironmentVariable("SDL_VIDEODRIVER");
        try
        {
            Environment.SetEnvironmentVariable("SDL_VIDEODRIVER", "dummy");
            using var window = new WindowManager(
                "Monogatari Headless Probe",
                64,
                64,
                RendererRuntimeMode.Headless);
            Assert.True(window.Initialize(), SdlError("SDL headless window initialization failed"));
            Assert.NotEqual(IntPtr.Zero, window.Window);
            SDL_GetWindowSize(window.Window, out var width, out var height);
            Assert.Equal(64, width);
            Assert.Equal(64, height);

            using var context = new RenderContext(window.Window, RendererRuntimeMode.Headless);
            Assert.NotEqual(IntPtr.Zero, context.Renderer);

            SDL_ClearError();
            for (var frame = 0; frame < 3; frame++)
            {
                context.Clear(8, 12, 18, 255);
                context.FillRect(8 + frame, 8, 24, 20, 30, 170, 120, 255);
                context.DrawRect(4, 4, 56, 56, 230, 235, 240, 255);
                context.DrawLine(0, frame, 63, 63 - frame, 245, 190, 70, 255);
                context.Present();

                while (SDL_PollEvent(out _) > 0)
                {
                }
            }

            Assert.Equal(string.Empty, Marshal.PtrToStringUTF8(SDL_GetError()) ?? string.Empty);
        }
        finally
        {
            Environment.SetEnvironmentVariable("SDL_VIDEODRIVER", previousVideoDriver);
        }
    }

    private static nint LoadWithExport(string fileName, string exportName)
    {
        var path = Path.Combine(AppContext.BaseDirectory, fileName);
        Assert.True(File.Exists(path), $"Missing renderer library: {fileName}");
        var handle = NativeLibrary.Load(path);
        Assert.True(NativeLibrary.TryGetExport(handle, exportName, out _),
            $"{fileName} does not export {exportName}");
        return handle;
    }

    private static string SdlError(string prefix)
    {
        var error = Marshal.PtrToStringUTF8(SDL_GetError());
        return string.IsNullOrWhiteSpace(error) ? prefix : $"{prefix}: {error}";
    }
}
