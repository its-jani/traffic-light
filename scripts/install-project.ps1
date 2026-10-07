# ==============================================================================
# Traffic Status - Single Project Installer (PowerShell)
# Installs Traffic Status slash commands & hooks strictly to one specified project.
# ==============================================================================
param(
    [Parameter(Position=0)]
    [string]$ProjectPath = "."
)

if (-not (Test-Path $ProjectPath)) {
    New-Item -ItemType Directory -Path $ProjectPath -Force | Out-Null
}
$targetDir = (Resolve-Path $ProjectPath).Path

Write-Host "🚦 Installing Traffic Status for project: $targetDir" -ForegroundColor Cyan

# 1. Claude Code Project Setup (.claude/commands/traffic.md)
$claudeDir = Join-Path $targetDir ".claude\commands"
if (-not (Test-Path $claudeDir)) {
    New-Item -ItemType Directory -Path $claudeDir -Force | Out-Null
}

$claudeMdContent = @'
<!-- generated-by: traffic-status -->
# Traffic Status Slash Command (/traffic)

Execute the corresponding action immediately in 1 step without reading source files or repository searches.

### Control & Auto-Start:

- **`/traffic on`** (or `/traffic`):
  Starts the floating status daemon (if not already running) and registers the active session.
  ```powershell
  traffic-status on "Claude"
  ```

- **`/traffic yellow [msg]`** (or when processing/thinking/running tools):
  ```powershell
  traffic-status yellow "$ARG"
  ```

- **`/traffic green [msg]`** (or when ready/waiting for prompt):
  ```powershell
  traffic-status green "$ARG"
  ```

- **`/traffic red [msg]`** (or when error/needs confirmation):
  ```powershell
  traffic-status red "$ARG"
  ```

- **`/traffic off`**:
  ```powershell
  traffic-status off
  ```
'@

Set-Content -Path (Join-Path $claudeDir "traffic.md") -Value $claudeMdContent -Encoding UTF8
Write-Host "  ✅ Added Claude Code command: .claude/commands/traffic.md" -ForegroundColor Green

# 2. OpenCode Project Setup (.opencode/plugins and .opencode/commands)
$opencodeCommandsDir = Join-Path $targetDir ".opencode\commands"
$opencodePluginsDir = Join-Path $targetDir ".opencode\plugins"

if (-not (Test-Path $opencodeCommandsDir)) {
    New-Item -ItemType Directory -Path $opencodeCommandsDir -Force | Out-Null
}
if (-not (Test-Path $opencodePluginsDir)) {
    New-Item -ItemType Directory -Path $opencodePluginsDir -Force | Out-Null
}

$opencodeMdContent = @'
<!-- generated-by: traffic-status -->
# Traffic Status Command (/traffic)

When `/traffic` is invoked in OpenCode, execute the corresponding action immediately in 1 step. **Do not search the repository, do not inspect source files, and do not run codebase searches.**

## Commands:
- `/traffic on` -> `traffic-status on "OpenCode"`
- `/traffic off` -> `traffic-status off`
- `/traffic yellow [message]` -> `traffic-status yellow "$ARG"`
- `/traffic green [message]` -> `traffic-status green "$ARG"`
- `/traffic red [message]` -> `traffic-status red "$ARG"`

### Automatic Lifecycle Rule:
- When thinking, compiling, or executing tools: send `yellow`.
- When an error occurs or waiting for user confirmation: send `red`.
- When task is completed and waiting for user prompt: send `green`.
'@

Set-Content -Path (Join-Path $opencodeCommandsDir "traffic.md") -Value $opencodeMdContent -Encoding UTF8
Write-Host "  ✅ Added OpenCode command: .opencode/commands/traffic.md" -ForegroundColor Green

$sourceRoot = Split-Path -Parent $PSScriptRoot
$pluginSrc = Join-Path $sourceRoot ".opencode\plugins\traffic-status.js"
if (Test-Path $pluginSrc) {
    Copy-Item -Path $pluginSrc -Destination (Join-Path $opencodePluginsDir "traffic-status.js") -Force
    Write-Host "  ✅ Added OpenCode plugin: .opencode/plugins/traffic-status.js" -ForegroundColor Green
}

# Clean legacy project file if present
$oldPlugin = Join-Path $opencodePluginsDir "traffic-light.js"
if (Test-Path $oldPlugin) {
    Remove-Item -Path $oldPlugin -Force -ErrorAction SilentlyContinue
}

Write-Host "`n✨ Project installation complete!" -ForegroundColor Green
Write-Host "Scope: Project-only ($targetDir)" -ForegroundColor DarkGray
Write-Host "You can now use /traffic on, /traffic off, etc. inside this project." -ForegroundColor Yellow
