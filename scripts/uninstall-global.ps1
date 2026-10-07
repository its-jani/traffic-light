# ==============================================================================
# Traffic Light - Complete Machine Uninstaller (PowerShell)
# Completely purges Traffic Light from the entire laptop (all global configs, binaries, & daemons).
# ==============================================================================
Write-Host "🚨 Completely removing Traffic Light from this laptop..." -ForegroundColor Red

# 1. Stop any running traffic-light processes
Write-Host "🛑 [1/4] Stopping any active Traffic Light instances..." -ForegroundColor Yellow
$running = Get-Process -Name "traffic-light" -ErrorAction SilentlyContinue
if ($running) {
    Stop-Process -Name "traffic-light" -Force -ErrorAction SilentlyContinue
    Write-Host "  ✅ Terminated running processes." -ForegroundColor Green
} else {
    Write-Host "  ⚪ No active process found." -ForegroundColor DarkGray
}

# 2. Uninstall binary from Cargo / System
Write-Host "📦 [2/4] Removing installed binaries..." -ForegroundColor Yellow
try {
    cargo uninstall traffic-light 2>$null | Out-Null
} catch {}

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin\traffic-light.exe"
if (Test-Path $cargoBin) {
    Remove-Item -Path $cargoBin -Force -ErrorAction SilentlyContinue
    Write-Host "  🗑️  Deleted $cargoBin" -ForegroundColor DarkYellow
}

# 3. Remove Global Claude Code commands
Write-Host "🤖 [3/4] Removing Global Claude Code configs..." -ForegroundColor Yellow
$globalClaude = Join-Path $env:USERPROFILE ".claude\commands\traffic.md"
if (Test-Path $globalClaude) {
    Remove-Item -Path $globalClaude -Force -ErrorAction SilentlyContinue
    Write-Host "  🗑️  Deleted $globalClaude" -ForegroundColor DarkYellow
}

# 4. Remove Global OpenCode plugins and commands
Write-Host "⚡ [4/4] Removing Global OpenCode configs..." -ForegroundColor Yellow
$globalOpenCodeDirs = @(
    (Join-Path $env:USERPROFILE ".config\opencode"),
    (Join-Path $env:USERPROFILE ".opencode")
)

foreach ($base in $globalOpenCodeDirs) {
    $cmd = Join-Path $base "commands\traffic.md"
    $plugin = Join-Path $base "plugins\traffic-light.js"
    if (Test-Path $cmd) {
        Remove-Item -Path $cmd -Force -ErrorAction SilentlyContinue
        Write-Host "  🗑️  Deleted $cmd" -ForegroundColor DarkYellow
    }
    if (Test-Path $plugin) {
        Remove-Item -Path $plugin -Force -ErrorAction SilentlyContinue
        Write-Host "  🗑️  Deleted $plugin" -ForegroundColor DarkYellow
    }
}

Write-Host "`n✨ Traffic Light has been completely purged from your system." -ForegroundColor Green
Write-Host "Scope: Entire Laptop (0 traces remaining)." -ForegroundColor DarkGray
