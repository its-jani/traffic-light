# ==============================================================================
# Traffic Light - Single Project Uninstaller (PowerShell)
# Removes Traffic Light slash commands & hooks strictly from one specified project.
# ==============================================================================
param(
    [Parameter(Position=0)]
    [string]$ProjectPath = "."
)

if (-not (Test-Path $ProjectPath)) {
    Write-Host "Project path '$ProjectPath' does not exist." -ForegroundColor Yellow
    exit 0
}
$targetDir = (Resolve-Path $ProjectPath).Path

Write-Host "🗑️  Removing Traffic Light from project: $targetDir" -ForegroundColor Cyan

# 1. Remove Claude Code command
$claudeFile = Join-Path $targetDir ".claude\commands\traffic.md"
if (Test-Path $claudeFile) {
    Remove-Item -Path $claudeFile -Force
    Write-Host "  🗑️  Removed .claude/commands/traffic.md" -ForegroundColor DarkYellow
    $claudeDir = Join-Path $targetDir ".claude\commands"
    if ((Get-ChildItem -Path $claudeDir -Force | Measure-Object).Count -eq 0) {
        Remove-Item -Path $claudeDir -Force -Recurse -ErrorAction SilentlyContinue
    }
}

# 2. Remove OpenCode command & plugin
$opencodeCmd = Join-Path $targetDir ".opencode\commands\traffic.md"
$opencodePlugin = Join-Path $targetDir ".opencode\plugins\traffic-light.js"

if (Test-Path $opencodeCmd) {
    Remove-Item -Path $opencodeCmd -Force
    Write-Host "  🗑️  Removed .opencode/commands/traffic.md" -ForegroundColor DarkYellow
}
if (Test-Path $opencodePlugin) {
    Remove-Item -Path $opencodePlugin -Force
    Write-Host "  🗑️  Removed .opencode/plugins/traffic-light.js" -ForegroundColor DarkYellow
}

# Clean empty directories if any
$opencodeCmdDir = Join-Path $targetDir ".opencode\commands"
$opencodePluginDir = Join-Path $targetDir ".opencode\plugins"
if ((Test-Path $opencodeCmdDir) -and (Get-ChildItem -Path $opencodeCmdDir -Force | Measure-Object).Count -eq 0) {
    Remove-Item -Path $opencodeCmdDir -Force -Recurse -ErrorAction SilentlyContinue
}
if ((Test-Path $opencodePluginDir) -and (Get-ChildItem -Path $opencodePluginDir -Force | Measure-Object).Count -eq 0) {
    Remove-Item -Path $opencodePluginDir -Force -Recurse -ErrorAction SilentlyContinue
}

Write-Host "`n✅ Project uninstallation complete!" -ForegroundColor Green
Write-Host "Traffic Light has been removed from $targetDir." -ForegroundColor DarkGray
