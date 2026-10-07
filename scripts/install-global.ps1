# ==============================================================================
# Traffic Status - Global System-Wide Installer (PowerShell)
# Installs Traffic Status globally across the entire machine for all current and future projects.
# ==============================================================================
$ErrorActionPreference = "Stop"

Write-Host "🚦 Installing Traffic Status Globally (All Projects)..." -ForegroundColor Cyan

$sourceRoot = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path (Join-Path $sourceRoot "Cargo.toml"))) {
    $sourceRoot = (Get-Location).Path
}

# 1. Build and install native Rust binary to ~/.cargo/bin & stable local app data
Write-Host "📦 [1/3] Installing native binary..." -ForegroundColor Yellow

$stableBinDir = Join-Path $env:LOCALAPPDATA "traffic-status\bin"
if (-not (Test-Path $stableBinDir)) {
    New-Item -ItemType Directory -Path $stableBinDir -Force | Out-Null
}

$builtExe = Join-Path $sourceRoot "target\release\traffic-status.exe"
if (-not (Test-Path $builtExe)) {
    $builtExe = Join-Path $sourceRoot "target\debug\traffic-status.exe"
}

if (Test-Path $builtExe) {
    Copy-Item -Path $builtExe -Destination (Join-Path $stableBinDir "traffic-status.exe") -Force
    Write-Host "  ✅ Installed binary to: $stableBinDir\traffic-status.exe" -ForegroundColor Green
} else {
    Push-Location $sourceRoot
    try {
        cargo build --release
        Copy-Item -Path (Join-Path $sourceRoot "target\release\traffic-status.exe") -Destination (Join-Path $stableBinDir "traffic-status.exe") -Force
        Write-Host "  ✅ Built and installed binary to: $stableBinDir\traffic-status.exe" -ForegroundColor Green
    } finally {
        Pop-Location
    }
}

# Also ensure on PATH if possible
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$stableBinDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$stableBinDir", "User")
    $env:PATH = "$env:PATH;$stableBinDir"
    Write-Host "  ✅ Added $stableBinDir to User PATH" -ForegroundColor Green
}

# 2. Global Claude Code Setup (~/.claude/commands/traffic.md)
Write-Host "🤖 [2/3] Configuring Global Claude Code commands..." -ForegroundColor Yellow
$globalClaudeDir = Join-Path $env:USERPROFILE ".claude\commands"
if (-not (Test-Path $globalClaudeDir)) {
    New-Item -ItemType Directory -Path $globalClaudeDir -Force | Out-Null
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

Set-Content -Path (Join-Path $globalClaudeDir "traffic.md") -Value $claudeMdContent -Encoding UTF8
Write-Host "  ✅ Global Claude Code command installed: $globalClaudeDir\traffic.md" -ForegroundColor Green

# 3. Global OpenCode Setup (~/.config/opencode & ~/.opencode)
Write-Host "⚡ [3/3] Configuring Global OpenCode plugins & commands..." -ForegroundColor Yellow
$globalOpenCodeTargets = @(
    (Join-Path $env:USERPROFILE ".config\opencode"),
    (Join-Path $env:USERPROFILE ".opencode")
)

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

$pluginSrc = Join-Path $sourceRoot ".opencode\plugins\traffic-status.js"

foreach ($baseDir in $globalOpenCodeTargets) {
    $cmdDir = Join-Path $baseDir "commands"
    $pluginDir = Join-Path $baseDir "plugins"

    if (-not (Test-Path $cmdDir)) { New-Item -ItemType Directory -Path $cmdDir -Force | Out-Null }
    if (-not (Test-Path $pluginDir)) { New-Item -ItemType Directory -Path $pluginDir -Force | Out-Null }

    Set-Content -Path (Join-Path $cmdDir "traffic.md") -Value $opencodeMdContent -Encoding UTF8
    if (Test-Path $pluginSrc) {
        Copy-Item -Path $pluginSrc -Destination (Join-Path $pluginDir "traffic-status.js") -Force
    }

    # Clean legacy plugin in global opencode
    $oldPlugin = Join-Path $pluginDir "traffic-light.js"
    if (Test-Path $oldPlugin) {
        Remove-Item -Path $oldPlugin -Force -ErrorAction SilentlyContinue
    }

    Write-Host "  ✅ Global OpenCode plugin & commands installed in: $baseDir" -ForegroundColor Green
}

# Clean legacy traffic-light binary if present in .cargo/bin
$oldCargoBin = Join-Path $env:USERPROFILE ".cargo\bin\traffic-light.exe"
if (Test-Path $oldCargoBin) {
    Remove-Item -Path $oldCargoBin -Force -ErrorAction SilentlyContinue
}

Write-Host "`n🎉 Global Installation Complete!" -ForegroundColor Green
Write-Host "Scope: Entire Laptop / All Projects (Current and Future)" -ForegroundColor Cyan
Write-Host "You can now run '/traffic on' from any folder or project in Claude Code or OpenCode." -ForegroundColor Yellow
