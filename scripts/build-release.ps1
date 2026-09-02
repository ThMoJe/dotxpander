<#
.SYNOPSIS
    Builds optimized release binaries for dotXPANDER using cargo build-std and UPX compression.

.DESCRIPTION
    1. Recompiles std with -Z build-std=std,panic_abort for minimal binary size.
    2. Strips dead code and unused symbols using MSVC linker flags.
    3. If UPX is installed, compresses x64 binaries down to ~3.3 MB (UPX does not support ARM64 PE).

.PARAMETER Target
    Target triple (default: current host architecture).
    Examples: x86_64-pc-windows-msvc, aarch64-pc-windows-msvc

.PARAMETER SkipUpx
    Skip UPX compression even if UPX is installed.
#>

param (
    [string]$Target = "",
    [switch]$SkipUpx
)

$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "             dotXPANDER — Optimized Release Builder         " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# 1. Check Nightly Toolchain & rust-src
Write-Host "`n[1/3] Checking toolchain requirements..." -ForegroundColor Yellow
$hasNightly = rustup toolchain list | Select-String "nightly"
if (-not $hasNightly) {
    Write-Host "Nightly toolchain not found. Installing nightly..." -ForegroundColor Gray
    rustup toolchain install nightly --profile minimal --component rust-src
}

# 2. Build with build-std
Write-Host "`n[2/3] Building release binary with build-std..." -ForegroundColor Yellow
$cargoArgs = @("+nightly", "build", "-Z", "build-std=std,panic_abort", "--release")
if ($Target) {
    $cargoArgs += @("--target", $Target)
}

& cargo @cargoArgs
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed with code $LASTEXITCODE"
}

# Locate output executable
$outputExe = if ($Target) { "target\$Target\release\dotxpander.exe" } else { "target\release\dotxpander.exe" }
if (-not (Test-Path $outputExe)) {
    Write-Error "Could not find built executable at $outputExe"
}

$origSize = (Get-Item $outputExe).Length
$origMb = [math]::Round($origSize / 1MB, 2)
Write-Host "Built: $outputExe ($origMb MB / $origSize bytes)" -ForegroundColor Green

# 3. UPX Compression (x64 only)
Write-Host "`n[3/3] Checking UPX compression..." -ForegroundColor Yellow
$isArm64 = ($Target -like "*aarch64*") -or (-not $Target -and ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq "Arm64"))

if ($isArm64) {
    Write-Host "Skipping UPX: Windows ARM64 PE binaries are not supported by UPX." -ForegroundColor DarkYellow
} elseif ($SkipUpx) {
    Write-Host "Skipping UPX: -SkipUpx flag specified." -ForegroundColor Gray
} else {
    $upxCmd = Get-Command "upx" -ErrorAction SilentlyContinue
    if ($upxCmd) {
        Write-Host "Running UPX compression..." -ForegroundColor Gray
        & upx --best --lzma $outputExe
        $newSize = (Get-Item $outputExe).Length
        $newMb = [math]::Round($newSize / 1MB, 2)
        $ratio = [math]::Round(($newSize / $origSize) * 100, 1)
        Write-Host "UPX Packed: $newMb MB ($newSize bytes, $ratio% of original)" -ForegroundColor Green
    } else {
        Write-Host "UPX not found in PATH. Binary left uncompressed." -ForegroundColor Gray
        Write-Host "Tip: Install UPX via 'winget install UPX.UPX' to enable 60% size compression." -ForegroundColor DarkGray
    }
}

Write-Host "`nRelease build complete!" -ForegroundColor Cyan
