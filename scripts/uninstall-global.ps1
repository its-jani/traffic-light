# ==============================================================================
# Traffic Status - Complete Machine Uninstaller (PowerShell)
# Removes Traffic Status from this machine (global configs, binaries, & daemons).
# ==============================================================================
Write-Host "🚨 Removing Traffic Status from this machine..." -ForegroundColor Red

# 1. Stop any running traffic-status processes
Write-Host "🛑 [1/4] Stopping any active Traffic Status instances..." -ForegroundColor Yellow
$running = Get-Process -Name "traffic-status" -ErrorAction SilentlyContinue
if ($running) {
    Stop-Process -Name "traffic-status" -Force -ErrorAction SilentlyContinue
    Write-Host "  ✅ Terminated running processes." -ForegroundColor Green
} else {
    Write-Host "  ⚪ No active process found." -ForegroundColor DarkGray
}

# 2. Remove stable binary
Write-Host "📦 [2/4] Removing installed binaries..." -ForegroundColor Yellow
$stableBin = Join-Path $env:LOCALAPPDATA "traffic-status\bin\traffic-status.exe"
if (Test-Path $stableBin) {
    Remove-Item -Path $stableBin -Force -ErrorAction SilentlyContinue
    Write-Host "  🗑️  Deleted $stableBin" -ForegroundColor DarkYellow
    $stableBinDir = Join-Path $env:LOCALAPPDATA "traffic-status\bin"
    if ((Test-Path $stableBinDir) -and (Get-ChildItem -Path $stableBinDir -Force | Measure-Object).Count -eq 0) {
        Remove-Item -Path $stableBinDir -Force -Recurse -ErrorAction SilentlyContinue
    }
}

$oldCargoBin = Join-Path $env:USERPROFILE ".cargo\bin\traffic-light.exe"
if (Test-Path $oldCargoBin) {
    Remove-Item -Path $oldCargoBin -Force -ErrorAction SilentlyContinue
}

# 3. Remove Global Claude Code commands if marked
Write-Host "🤖 [3/4] Removing Global Claude Code configs..." -ForegroundColor Yellow
$globalClaude = Join-Path $env:USERPROFILE ".claude\commands\traffic.md"
if (Test-Path $globalClaude) {
    $content = Get-Content $globalClaude -Raw -ErrorAction SilentlyContinue
    if ($content -match "traffic-status" -or $content -match "traffic-light") {
        Remove-Item -Path $globalClaude -Force -ErrorAction SilentlyContinue
        Write-Host "  🗑️  Deleted $globalClaude" -ForegroundColor DarkYellow
    }
}

# 4. Remove Global OpenCode plugins and commands (files only, never destroy whole dir)
Write-Host "⚡ [4/4] Removing Global OpenCode configs..." -ForegroundColor Yellow
$globalOpenCodeDirs = @(
    (Join-Path $env:USERPROFILE ".config\opencode"),
    (Join-Path $env:USERPROFILE ".opencode")
)

foreach ($base in $globalOpenCodeDirs) {
    $cmd = Join-Path $base "commands\traffic.md"
    $plugin = Join-Path $base "plugins\traffic-status.js"
    $oldPlugin = Join-Path $base "plugins\traffic-light.js"

    if (Test-Path $cmd) {
        Remove-Item -Path $cmd -Force -ErrorAction SilentlyContinue
        Write-Host "  🗑️  Deleted $cmd" -ForegroundColor DarkYellow
    }
    if (Test-Path $plugin) {
        Remove-Item -Path $plugin -Force -ErrorAction SilentlyContinue
        Write-Host "  🗑️  Deleted $plugin" -ForegroundColor DarkYellow
    }
    if (Test-Path $oldPlugin) {
        Remove-Item -Path $oldPlugin -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "`n✨ Traffic Status global uninstallation complete." -ForegroundColor Green
