# Build and package the native GPUI executable, without the old Tauri toolchain.
param(
    [ValidateSet('x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc')]
    [string]$Target = 'x86_64-pc-windows-msvc'
)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)

$versionMatch = Select-String -Path Cargo.toml -Pattern '^version = "([^"]+)"' | Select-Object -First 1
if (-not $versionMatch) { throw 'Package version was not found in Cargo.toml' }
$version = $versionMatch.Matches[0].Groups[1].Value
$arch = if ($Target -eq 'aarch64-pc-windows-msvc') { 'arm64' } else { 'x64' }
# Keep portable downloads independent of a separately installed MSVC runtime.
if (-not $env:RUSTFLAGS) { $env:RUSTFLAGS = '-C target-feature=+crt-static' }
cargo build --release --locked --target $Target
if ($LASTEXITCODE -ne 0) { throw "cargo build failed: $LASTEXITCODE" }

$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
$exe = Join-Path $targetDir "$Target/release/refiner.exe"
if (-not (Test-Path $exe)) { throw "Missing release executable: $exe" }
New-Item -ItemType Directory -Force -Path dist | Out-Null
$stage = Join-Path ([IO.Path]::GetTempPath()) "refiner-package-$([Guid]::NewGuid())"
New-Item -ItemType Directory -Path $stage | Out-Null
try {
    Copy-Item $exe (Join-Path $stage 'Refiner.exe')
    Copy-Item LICENSE $stage
    Copy-Item README.md $stage
    $archive = "dist/Refiner_${version}_windows_${arch}.zip"
    Compress-Archive -Path "$stage/*" -DestinationPath $archive -Force
    Write-Output "Packaged $archive"
} finally {
    Remove-Item $stage -Recurse -Force
}
