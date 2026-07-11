using System.Runtime.InteropServices;

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

    private static nint LoadWithExport(string fileName, string exportName)
    {
        var path = Path.Combine(AppContext.BaseDirectory, fileName);
        Assert.True(File.Exists(path), $"Missing renderer library: {fileName}");
        var handle = NativeLibrary.Load(path);
        Assert.True(NativeLibrary.TryGetExport(handle, exportName, out _),
            $"{fileName} does not export {exportName}");
        return handle;
    }
}
