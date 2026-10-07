# ==============================================================================
# Traffic Light - Global System-Wide Installer (PowerShell)
# Installs Traffic Light globally across the entire machine for all current and future projects.
# ==============================================================================
$ErrorActionPreference = "Stop"

Write-Host "🚦 Installing Traffic Light Globally (All Projects)..." -ForegroundColor Cyan

$sourceRoot = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path (Join-Path $sourceRoot "Cargo.toml"))) {
    $sourceRoot = (Get-Location).Path
}

# 1. Build and install native Rust binary to ~/.cargo/bin
Write-Host "📦 [1/3] Building & installing native binary (cargo install)..." -ForegroundColor Yellow
Push-Location $sourceRoot
try {
    cargo install --path . --force
    Write-Host "  ✅ Binary installed to: $env:USERPROFILE\.cargo\bin\traffic-light.exe" -ForegroundColor Green
} catch {
    Write-Warning "Cargo install encountered an issue: $_. Checking existing binary in target\release..."
    $localExe = Join-Path $sourceRoot "target\release\traffic-light.exe"
    $cargoBinDir = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $localExe) {
        if (-not (Test-Path $cargoBinDir)) { New-Item -ItemType Directory -Path $cargoBinDir -Force | Out-Null }
        Copy-Item -Path $localExe -Destination (Join-Path $cargoBinDir "traffic-light.exe") -Force
        Write-Host "  ✅ Copied release binary to: $cargoBinDir\traffic-light.exe" -ForegroundColor Green
    }
} finally {
    Pop-Location
}

# 2. Global Claude Code Setup (~/.claude/commands/traffic.md)
Write-Host "🤖 [2/3] Configuring Global Claude Code commands..." -ForegroundColor Yellow
$globalClaudeDir = Join-Path $env:USERPROFILE ".claude\commands"
if (-not (Test-Path $globalClaudeDir)) {
    New-Item -ItemType Directory -Path $globalClaudeDir -Force | Out-Null
}
$claudeSrc = Join-Path $sourceRoot ".claude\commands\traffic.md"
if (Test-Path $claudeSrc) {
    Copy-Item -Path $claudeSrc -Destination (Join-Path $globalClaudeDir "traffic.md") -Force
    Write-Host "  ✅ Global Claude Code command installed: $globalClaudeDir\traffic.md" -ForegroundColor Green
}

# 3. Global OpenCode Setup (~/.config/opencode & ~/.opencode)
Write-Host "⚡ [3/3] Configuring Global OpenCode plugins & commands..." -ForegroundColor Yellow
$globalOpenCodeTargets = @(
    (Join-Path $env:USERPROFILE ".config\opencode"),
    (Join-Path $env:USERPROFILE ".opencode")
)

$opencodeCmdSrc = Join-Path $sourceRoot ".opencode\commands\traffic.md"
$opencodePluginSrc = Join-Path $sourceRoot ".opencode\plugins\traffic-light.js"

foreach ($baseDir in $globalOpenCodeTargets) {
    $cmdDir = Join-Path $baseDir "commands"
    $pluginDir = Join-Path $baseDir "plugins"

    if (-not (Test-Path $cmdDir)) { New-Item -ItemType Directory -Path $cmdDir -Force | Out-Null }
    if (-not (Test-Path $pluginDir)) { New-Item -ItemType Directory -Path $pluginDir -Force | Out-Null }

    if (Test-Path $opencodeCmdSrc) {
        Copy-Item -Path $opencodeCmdSrc -Destination (Join-Path $cmdDir "traffic.md") -Force
    }
    if (Test-Path $opencodePluginSrc) {
        Copy-Item -Path $opencodePluginSrc -Destination (Join-Path $pluginDir "traffic-light.js") -Force
    }
    Write-Host "  ✅ Global OpenCode plugin & commands installed in: $baseDir" -ForegroundColor Green
}

Write-Host "`n🎉 Global Installation Complete!" -ForegroundColor Green
Write-Host "Scope: Entire Laptop / All Projects (Current and Future)" -ForegroundColor Cyan
Write-Host "You can now run '/traffic on' from any folder or project in Claude Code or OpenCode." -ForegroundColor Yellow
