# ==============================================================================
# Traffic Light - Single Project Installer (PowerShell)
# Installs Traffic Light slash commands & hooks strictly to one specified project.
# ==============================================================================
param(
    [Parameter(Position=0)]
    [string]$ProjectPath = "."
)

if (-not (Test-Path $ProjectPath)) {
    New-Item -ItemType Directory -Path $ProjectPath -Force | Out-Null
}
$targetDir = (Resolve-Path $ProjectPath).Path

Write-Host "🚦 Installing Traffic Light for project: $targetDir" -ForegroundColor Cyan

$sourceRoot = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path (Join-Path $sourceRoot "Cargo.toml"))) {
    $sourceRoot = (Get-Location).Path
}

# 1. Claude Code Project Setup (.claude/commands/traffic.md)
$claudeDir = Join-Path $targetDir ".claude\commands"
if (-not (Test-Path $claudeDir)) {
    New-Item -ItemType Directory -Path $claudeDir -Force | Out-Null
}
$claudeSrc = Join-Path $sourceRoot ".claude\commands\traffic.md"
if (Test-Path $claudeSrc) {
    Copy-Item -Path $claudeSrc -Destination (Join-Path $claudeDir "traffic.md") -Force
    Write-Host "  ✅ Added Claude Code command: .claude/commands/traffic.md" -ForegroundColor Green
}

# 2. OpenCode Project Setup (.opencode/plugins and .opencode/commands)
$opencodeCommandsDir = Join-Path $targetDir ".opencode\commands"
$opencodePluginsDir = Join-Path $targetDir ".opencode\plugins"

if (-not (Test-Path $opencodeCommandsDir)) {
    New-Item -ItemType Directory -Path $opencodeCommandsDir -Force | Out-Null
}
if (-not (Test-Path $opencodePluginsDir)) {
    New-Item -ItemType Directory -Path $opencodePluginsDir -Force | Out-Null
}

$opencodeCmdSrc = Join-Path $sourceRoot ".opencode\commands\traffic.md"
$opencodePluginSrc = Join-Path $sourceRoot ".opencode\plugins\traffic-light.js"

if (Test-Path $opencodeCmdSrc) {
    Copy-Item -Path $opencodeCmdSrc -Destination (Join-Path $opencodeCommandsDir "traffic.md") -Force
    Write-Host "  ✅ Added OpenCode command: .opencode/commands/traffic.md" -ForegroundColor Green
}
if (Test-Path $opencodePluginSrc) {
    Copy-Item -Path $opencodePluginSrc -Destination (Join-Path $opencodePluginsDir "traffic-light.js") -Force
    Write-Host "  ✅ Added OpenCode plugin: .opencode/plugins/traffic-light.js" -ForegroundColor Green
}

Write-Host "`n✨ Project installation complete!" -ForegroundColor Green
Write-Host "Scope: Project-only ($targetDir)" -ForegroundColor DarkGray
Write-Host "You can now use /traffic on, /traffic off, etc. inside this project." -ForegroundColor Yellow
