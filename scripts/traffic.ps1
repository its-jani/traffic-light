<#
.SYNOPSIS
    Traffic Status CLI integration for PowerShell (OpenCode / Claude Code / Windows Terminal).
#>

$global:TrafficPort = if ($env:TRAFFIC_STATUS_PORT) { $env:TRAFFIC_STATUS_PORT } else { 8765 }
$global:TrafficHost = "127.0.0.1"
$global:TrafficSessionId = if ($env:TRAFFIC_SESSION_ID) { $env:TRAFFIC_SESSION_ID } else { "session-$PID" }

function Test-TrafficStatusRunning {
    $url = "http://${global:TrafficHost}:${global:TrafficPort}/ping"
    try {
        $res = Invoke-RestMethod -Uri $url -Method Get -TimeoutSec 1 -ErrorAction Stop
        return ($res.status -eq "ok")
    } catch {
        return $false
    }
}

function Ensure-TrafficStatusDaemon {
    if (Test-TrafficStatusRunning) {
        return $true
    }
    # Check PATH first
    $cmd = Get-Command "traffic-status" -ErrorAction SilentlyContinue
    if ($cmd) {
        Start-Process -FilePath $cmd.Source -WindowStyle Hidden
        Start-Sleep -Milliseconds 450
        return (Test-TrafficStatusRunning)
    }
    # Check stable bin
    $stableBin = Join-Path $env:LOCALAPPDATA "traffic-status\bin\traffic-status.exe"
    if (Test-Path $stableBin) {
        Start-Process -FilePath $stableBin -WindowStyle Hidden
        Start-Sleep -Milliseconds 450
        return (Test-TrafficStatusRunning)
    }
    # Check Cargo bin
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin\traffic-status.exe"
    if (Test-Path $cargoBin) {
        Start-Process -FilePath $cargoBin -WindowStyle Hidden
        Start-Sleep -Milliseconds 450
        return (Test-TrafficStatusRunning)
    }
    # Check local target folder
    $localBin = Join-Path $PSScriptRoot "..\target\release\traffic-status.exe"
    if (Test-Path $localBin) {
        Start-Process -FilePath $localBin -WindowStyle Hidden
        Start-Sleep -Milliseconds 450
        return (Test-TrafficStatusRunning)
    }
    return $false
}

function Send-TrafficCommand {
    param(
        [string]$Endpoint,
        [hashtable]$Body
    )
    $url = "http://${global:TrafficHost}:${global:TrafficPort}$Endpoint"
    try {
        $json = $Body | ConvertTo-Json -Compress
        Invoke-RestMethod -Uri $url -Method Post -Body $json -ContentType "application/json" -TimeoutSec 1 | Out-Null
    } catch {
        # Silently fail if traffic-status is not running
    }
}

function traffic {
    param(
        [Parameter(Position=0)]
        [string]$Action = "green",
        [Parameter(Position=1)]
        [string]$Message = "",
        [Parameter(Position=2)]
        [string]$SessionId = $global:TrafficSessionId
    )

    switch ($Action.ToLower()) {
        "on" {
            Ensure-TrafficStatusDaemon | Out-Null
            Send-TrafficCommand -Endpoint "/session/on" -Body @{
                session_id = $SessionId
                label = if ($Message) { $Message } else { "Session ($PID)" }
                state = "green"
            }
            Write-Host "[Traffic Status] Session '$SessionId' ON (Green)" -ForegroundColor Green
        }
        "off" {
            Send-TrafficCommand -Endpoint "/session/off" -Body @{
                session_id = $SessionId
            }
            Write-Host "[Traffic Status] Session '$SessionId' OFF" -ForegroundColor DarkGray
        }
        "yellow" {
            $msg = if ($Message) { $Message } else { "Thinking / Generating..." }
            Send-TrafficCommand -Endpoint "/state" -Body @{
                session_id = $SessionId
                state = "yellow"
                message = $msg
            }
        }
        "green" {
            $msg = if ($Message) { $Message } else { "Ready / Done" }
            Send-TrafficCommand -Endpoint "/state" -Body @{
                session_id = $SessionId
                state = "green"
                message = $msg
            }
        }
        "red" {
            $msg = if ($Message) { $Message } else { "Needs user input / Halted" }
            Send-TrafficCommand -Endpoint "/state" -Body @{
                session_id = $SessionId
                state = "red"
                message = $msg
            }
        }
        { $_ -in "install-project", "install_project" } {
            $script = Join-Path $PSScriptRoot "install-project.ps1"
            & $script $Message
        }
        { $_ -in "install-global", "install_global" } {
            $script = Join-Path $PSScriptRoot "install-global.ps1"
            & $script
        }
        { $_ -in "uninstall-project", "uninstall_project" } {
            $script = Join-Path $PSScriptRoot "uninstall-project.ps1"
            & $script $Message
        }
        { $_ -in "uninstall-global", "uninstall_global", "uninstall-all" } {
            $script = Join-Path $PSScriptRoot "uninstall-global.ps1"
            & $script
        }
        { $_ -in "wrap", "run" } {
            if ($Message) {
                traffic yellow "Running: $Message"
                try {
                    Invoke-Expression $Message
                    if ($LASTEXITCODE -eq 0 -or $null -eq $LASTEXITCODE) {
                        traffic green "Completed successfully"
                    } else {
                        traffic red "Failed (exit code $LASTEXITCODE)"
                    }
                } catch {
                    traffic red "Error: $_"
                }
            }
        }
        "clear" {
            Send-TrafficCommand -Endpoint "/clear" -Body @{}
            Write-Host "[Traffic Status] All sessions cleared." -ForegroundColor DarkYellow
        }
        default {
            Write-Host "🚦 Traffic Status CLI Controller" -ForegroundColor Cyan
            Write-Host "Session Controls:" -ForegroundColor Yellow
            Write-Host "  traffic on                        - Activate traffic status session"
            Write-Host "  traffic off                       - Dismiss traffic status session"
            Write-Host "  traffic green|yellow|red [msg]    - Update current status light"
            Write-Host "  traffic wrap '<cmd>'              - Automatically wrap command with traffic states"
            Write-Host "`nManagement Commands:" -ForegroundColor Yellow
            Write-Host "  traffic install-project [path]    - Install traffic status for a specific project"
            Write-Host "  traffic install-global            - Install traffic status globally for all projects"
            Write-Host "  traffic uninstall-project [path]  - Remove traffic status from a specific project"
            Write-Host "  traffic uninstall-global          - Remove traffic status from the machine"
        }
    }
}
