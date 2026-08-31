# PowerShell build automation script for Microchip PolarFire SoC Icicle Kit (Renode 64-Bit RISC-V Target)

param(
    [switch]$Clean,
    [switch]$Rebuild,
    [string]$BuildType = "Debug",
    [string]$ToolchainPath = ""
)

$ErrorActionPreference = "Stop"
$TargetDir = Split-Path -Parent $PSScriptRoot
$BuildDir = Join-Path $TargetDir "build"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " Building PolarFire SoC Icicle Kit Target  " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Dynamic Toolchain Path Detection
if ($ToolchainPath -ne "") {
    $env:RISCV64_TOOLCHAIN_PATH = $ToolchainPath
} elseif (-not $env:RISCV64_TOOLCHAIN_PATH) {
    # Check default toolchain candidates in user profile
    $Candidates = @(
        (Join-Path $env:USERPROFILE "Downloads\xpack-riscv-none-elf-gcc-14.2.0-1-win32-x64\xpack-riscv-none-elf-gcc-14.2.0-1"),
        (Join-Path $env:USERPROFILE "Downloads\tools\xpack-riscv-none-elf-gcc-14.2.0-1"),
        (Join-Path $env:ProgramFiles "xpack-riscv-none-elf-gcc-14.2.0-1")
    )
    foreach ($Candidate in $Candidates) {
        if (Test-Path $Candidate) {
            $env:RISCV64_TOOLCHAIN_PATH = $Candidate
            break
        }
    }
}

# 2. Dynamic Ninja Tool Detection
$NinjaCandidates = @(
    (Join-Path $env:USERPROFILE "Downloads\tools\ninja-win"),
    (Join-Path $env:ProgramFiles "Ninja")
)
foreach ($NinjaPath in $NinjaCandidates) {
    if (Test-Path $NinjaPath) {
        $env:PATH = "$NinjaPath;" + $env:PATH
        break
    }
}

if ($env:RISCV64_TOOLCHAIN_PATH) {
    Write-Host "Using RISCV64_TOOLCHAIN_PATH: $env:RISCV64_TOOLCHAIN_PATH" -ForegroundColor Yellow
}

if ($Clean -or $Rebuild) {
    if (Test-Path $BuildDir) {
        Write-Host "Cleaning build directory..." -ForegroundColor Yellow
        Remove-Item -Path $BuildDir -Recurse -Force
    }
}

if (-not (Test-Path $BuildDir)) {
    New-Item -Path $BuildDir -ItemType Directory | Out-Null
}

Push-Location $BuildDir
try {
    Write-Host "Configuring CMake (Ninja Generator)..." -ForegroundColor Green
    cmake -G "Ninja" "-DCMAKE_BUILD_TYPE=$BuildType" ..

    Write-Host "Building target binaries..." -ForegroundColor Green
    ninja

    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host " Build Completed Successfully!           " -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Cyan
}
finally {
    Pop-Location
}
