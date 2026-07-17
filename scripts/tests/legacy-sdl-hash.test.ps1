Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot '..\lib\legacy-sdl-hash.ps1')

$fixturePath = Join-Path ([System.IO.Path]::GetTempPath()) "monogatari-sha256-$PID.fixture"
try {
    [System.IO.File]::WriteAllBytes($fixturePath, [byte[]](97, 98, 99))
    $actual = Get-LowerSha256 $fixturePath
    $expected = 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'
    if ($actual -cne $expected) {
        throw "Expected SHA-256 $expected, received $actual"
    }

    $exclusive = [System.IO.File]::Open(
        $fixturePath,
        [System.IO.FileMode]::Open,
        [System.IO.FileAccess]::ReadWrite,
        [System.IO.FileShare]::None
    )
    $exclusive.Dispose()
    Write-Host '[legacy-sdl-hash] 1 passed'
}
finally {
    Remove-Item -LiteralPath $fixturePath -Force -ErrorAction SilentlyContinue
}
