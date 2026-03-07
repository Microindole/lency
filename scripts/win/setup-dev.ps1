param(
    [switch]$Persist,
    [string]$LlvmPrefix = "",
    [string[]]$SearchRoots = @()
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Test-LlvmPrefix {
    param([Parameter(Mandatory = $true)][string]$Prefix)

    if (-not (Test-Path $Prefix)) {
        return $false
    }

    $llvmConfigNames = @("llvm-config.exe", "llvm-config-15.exe")
    $llvmConfig = $null
    foreach ($name in $llvmConfigNames) {
        $candidate = Join-Path $Prefix ("bin\\" + $name)
        if (Test-Path $candidate) {
            $llvmConfig = $candidate
            break
        }
    }
    if (-not $llvmConfig) {
        return $false
    }

    try {
        $version = & $llvmConfig --version
    } catch {
        return $false
    }

    return $version -like "15.*"
}

function Resolve-LlvmPrefix {
    param([string]$PreferredPrefix)

    if ($PreferredPrefix -and (Test-LlvmPrefix -Prefix $PreferredPrefix)) {
        return (Resolve-Path $PreferredPrefix).Path
    }

    if ($env:LLVM_SYS_150_PREFIX -and (Test-LlvmPrefix -Prefix $env:LLVM_SYS_150_PREFIX)) {
        return (Resolve-Path $env:LLVM_SYS_150_PREFIX).Path
    }

    $commands = @("llvm-config.exe", "llvm-config-15.exe")
    foreach ($cmd in $commands) {
        $llvmConfigOnPath = Get-Command $cmd -ErrorAction SilentlyContinue
        if ($llvmConfigOnPath) {
            $binDir = Split-Path -Parent $llvmConfigOnPath.Path
            $prefix = Split-Path -Parent $binDir
            if (Test-LlvmPrefix -Prefix $prefix) {
                return (Resolve-Path $prefix).Path
            }
        }
    }

    $roots = @()
    if ($SearchRoots.Count -gt 0) {
        $roots += $SearchRoots
    } else {
        foreach ($v in @($env:ProgramFiles, ${env:ProgramFiles(x86)}, $env:LOCALAPPDATA, $env:USERPROFILE)) {
            if ($v) { $roots += $v }
        }
        $drives = Get-PSDrive -PSProvider FileSystem -ErrorAction SilentlyContinue
        foreach ($d in $drives) {
            $root = $d.Root
            foreach ($candidate in @("${root}tools", "${root}llvm", "${root}dev")) {
                if (Test-Path $candidate) {
                    $roots += $candidate
                }
            }
        }
    }

    foreach ($root in ($roots | Select-Object -Unique)) {
        if (-not (Test-Path $root)) { continue }
        if (Test-LlvmPrefix -Prefix $root) {
            return (Resolve-Path $root).Path
        }
        try {
            $dirs = Get-ChildItem -Path $root -Directory -Depth 3 -ErrorAction SilentlyContinue |
                Where-Object { $_.Name -match "(?i)llvm" -or $_.FullName -match "(?i)\\LLVM" }
            foreach ($dir in $dirs) {
                if (Test-LlvmPrefix -Prefix $dir.FullName) {
                    return (Resolve-Path $dir.FullName).Path
                }
            }
        } catch {
            continue
        }
    }

    return $null
}

if (-not $IsWindows) {
    Write-Error "This script is for Windows. Use scripts/linux/setup-dev.sh on Linux/macOS."
}

$prefix = Resolve-LlvmPrefix -PreferredPrefix $LlvmPrefix
if (-not $prefix) {
    Write-Host "No usable LLVM 15 installation found." -ForegroundColor Red
    Write-Host "Expected llvm-config --version to start with 15.x." -ForegroundColor Yellow
    Write-Host "Install LLVM 15 and rerun:"
    Write-Host "  .\\scripts\\setup-dev.ps1 -LlvmPrefix <your-llvm-prefix> -Persist"
    exit 1
}

$env:LLVM_SYS_150_PREFIX = $prefix
$llvmConfig = $null
foreach ($name in @("llvm-config.exe", "llvm-config-15.exe")) {
    $candidate = Join-Path $prefix ("bin\\" + $name)
    if (Test-Path $candidate) {
        $llvmConfig = $candidate
        break
    }
}
$version = & $llvmConfig --version
$targets = & $llvmConfig --targets-built

Write-Host "Detected LLVM prefix: $prefix" -ForegroundColor Green
Write-Host "llvm-config version: $version"
Write-Host "targets-built: $targets"

if ($Persist) {
    [Environment]::SetEnvironmentVariable("LLVM_SYS_150_PREFIX", $prefix, "User")
    Write-Host "Saved LLVM_SYS_150_PREFIX to User environment." -ForegroundColor Green
} else {
    Write-Host "Set for current shell only. Add -Persist to save permanently." -ForegroundColor Yellow
}

$pathEntries = ($env:PATH -split ";")
$llvmBin = Join-Path $prefix "bin"
if (-not ($pathEntries -contains $llvmBin)) {
    Write-Host "Tip: add $llvmBin to PATH if llvm-config is not globally available." -ForegroundColor Yellow
}

Write-Host "Next step: cargo build -v"
