[CmdletBinding()]
param(
    [switch]$ForceDownload
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'lib\legacy-sdl-hash.ps1')

$isWindowsHost = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Windows
)
if (-not $isWindowsHost -or [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -ne 'X64') {
    throw 'The retained SDL2 application currently supports only Windows x64.'
}

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$cacheRoot = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot '.cache\legacy-sdl'))
$runtimeRoot = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot 'runtimes\win-x64\native'))
$rootPrefix = $repositoryRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar

function Assert-RepositoryPath {
    param([Parameter(Mandatory = $true)][string]$Path)

    $resolved = [System.IO.Path]::GetFullPath($Path)
    if (-not $resolved.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Generated path escapes the repository root: $resolved"
    }
    return $resolved
}

$archives = @(
    @{
        Label = 'SDL2 2.32.10'
        FileName = 'SDL2-2.32.10-win32-x64.zip'
        Url = 'https://github.com/libsdl-org/SDL/releases/download/release-2.32.10/SDL2-2.32.10-win32-x64.zip'
        Sha256 = '6cf9706eefd0a4a06dc764007934d428afaf029fabdd408a9e646048c91e18fb'
        Entries = @(
            @{ Source = 'SDL2.dll'; Target = 'SDL2.dll' },
            @{ Source = 'README-SDL.txt'; Target = 'licenses/SDL2-README.txt' }
        )
    },
    @{
        Label = 'SDL2_image 2.8.8'
        FileName = 'SDL2_image-2.8.8-win32-x64.zip'
        Url = 'https://github.com/libsdl-org/SDL_image/releases/download/release-2.8.8/SDL2_image-2.8.8-win32-x64.zip'
        Sha256 = '740d6c7fe40087db01e0d7bd636e3eaebe4bffbde58b595bf0c3b7c359a43033'
        Entries = @(
            @{ Source = 'SDL2_image.dll'; Target = 'SDL2_image.dll' },
            @{ Source = 'optional/libavif-16.dll'; Target = 'libavif-16.dll' },
            @{ Source = 'optional/libtiff-5.dll'; Target = 'libtiff-5.dll' },
            @{ Source = 'optional/libwebp-7.dll'; Target = 'libwebp-7.dll' },
            @{ Source = 'optional/libwebpdemux-2.dll'; Target = 'libwebpdemux-2.dll' },
            @{ Source = 'README.txt'; Target = 'licenses/SDL2_image-README.txt' },
            @{ Source = 'optional/LICENSE.avif.txt'; Target = 'licenses/SDL2_image-LICENSE.avif.txt' },
            @{ Source = 'optional/LICENSE.dav1d.txt'; Target = 'licenses/SDL2_image-LICENSE.dav1d.txt' },
            @{ Source = 'optional/LICENSE.tiff.txt'; Target = 'licenses/SDL2_image-LICENSE.tiff.txt' },
            @{ Source = 'optional/LICENSE.webp.txt'; Target = 'licenses/SDL2_image-LICENSE.webp.txt' }
        )
    },
    @{
        Label = 'SDL2_ttf 2.24.0'
        FileName = 'SDL2_ttf-2.24.0-win32-x64.zip'
        Url = 'https://github.com/libsdl-org/SDL_ttf/releases/download/release-2.24.0/SDL2_ttf-2.24.0-win32-x64.zip'
        Sha256 = '8d72240aafc783d37d6cf960b1a2138e6ca4eb0c1b5b9a9e71ea299d1ef27ef9'
        Entries = @(
            @{ Source = 'SDL2_ttf.dll'; Target = 'SDL2_ttf.dll' },
            @{ Source = 'README.txt'; Target = 'licenses/SDL2_ttf-README.txt' }
        )
    }
)

New-Item -ItemType Directory -Path (Assert-RepositoryPath $cacheRoot) -Force | Out-Null
New-Item -ItemType Directory -Path (Assert-RepositoryPath $runtimeRoot) -Force | Out-Null
Add-Type -AssemblyName System.IO.Compression.FileSystem

foreach ($archive in $archives) {
    $archivePath = Assert-RepositoryPath (Join-Path $cacheRoot $archive.FileName)
    $downloadPath = Assert-RepositoryPath ($archivePath + '.download')
    $downloadRequired = $ForceDownload -or -not (Test-Path -LiteralPath $archivePath -PathType Leaf)
    if (-not $downloadRequired -and (Get-LowerSha256 $archivePath) -ne $archive.Sha256) {
        $downloadRequired = $true
    }

    if ($downloadRequired) {
        Remove-Item -LiteralPath $downloadPath -Force -ErrorAction SilentlyContinue
        Write-Host "[legacy-sdl] Downloading $($archive.Label)"
        Invoke-WebRequest -UseBasicParsing -Uri $archive.Url -OutFile $downloadPath
        $downloadHash = Get-LowerSha256 $downloadPath
        if ($downloadHash -ne $archive.Sha256) {
            Remove-Item -LiteralPath $downloadPath -Force -ErrorAction SilentlyContinue
            throw "$($archive.Label) archive SHA-256 mismatch: $downloadHash"
        }
        Move-Item -LiteralPath $downloadPath -Destination $archivePath -Force
    }

    $archiveHash = Get-LowerSha256 $archivePath
    if ($archiveHash -ne $archive.Sha256) {
        throw "$($archive.Label) cached archive SHA-256 mismatch: $archiveHash"
    }

    $zip = [System.IO.Compression.ZipFile]::OpenRead($archivePath)
    try {
        foreach ($entryDefinition in $archive.Entries) {
            $entry = $zip.GetEntry($entryDefinition.Source)
            if ($null -eq $entry -or $entry.Length -le 0 -or $entry.Length -gt 64MB) {
                throw "$($archive.Label) has an invalid required entry: $($entryDefinition.Source)"
            }

            $targetPath = Assert-RepositoryPath (Join-Path $runtimeRoot $entryDefinition.Target)
            $targetParent = Assert-RepositoryPath (Split-Path $targetPath -Parent)
            New-Item -ItemType Directory -Path $targetParent -Force | Out-Null
            $stagePath = Assert-RepositoryPath (Join-Path $targetParent ".$(Split-Path $targetPath -Leaf).$PID.tmp")
            Remove-Item -LiteralPath $stagePath -Force -ErrorAction SilentlyContinue
            $sourceStream = $entry.Open()
            $targetStream = [System.IO.File]::Open(
                $stagePath,
                [System.IO.FileMode]::CreateNew,
                [System.IO.FileAccess]::Write,
                [System.IO.FileShare]::None
            )
            try {
                $sourceStream.CopyTo($targetStream)
                $targetStream.Flush($true)
            }
            finally {
                $targetStream.Dispose()
                $sourceStream.Dispose()
            }
            Move-Item -LiteralPath $stagePath -Destination $targetPath -Force
            Write-Host "[legacy-sdl] Ready $($entryDefinition.Target) ($($entry.Length) bytes)"
        }
    }
    finally {
        $zip.Dispose()
    }
}

Write-Host "[legacy-sdl] Windows x64 runtime ready at $runtimeRoot"
