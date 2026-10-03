<#
.SYNOPSIS
    Traffic Light CLI integration for PowerShell (OpenCode / Claude Code / Windows Terminal).
#>

$global:TrafficPort = 8765
$global:TrafficHost = "127.0.0.1"
$global:TrafficSessionId = if ($env:TRAFFIC_SESSION_ID) { $env:TRAFFIC_SESSION_ID } else { "session-$PID" }

function Test-TrafficLightRunning {
    $url = "http://${global:TrafficHost}:${global:TrafficPort}/ping"
    try {
        $res = Invoke-RestMethod -Uri $url -Method Get -TimeoutSec 1 -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

function Ensure-TrafficLightDaemon {
    if (Test-TrafficLightRunning) {
        return $true
    }
    # Check PATH first
    $cmd = Get-Command "traffic-light" -ErrorAction SilentlyContinue
    if ($cmd) {
        Start-Process -FilePath $cmd.Source -WindowStyle Hidden
        Start-Sleep -Milliseconds 450
        return (Test-TrafficLightRunning)
    }
    # Check Cargo bin
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin\traffic-light.exe"
    if (Test-Path $cargoBin) {
        Start-Process -FilePath $cargoBin -WindowStyle Hidden
        Start-Sleep -Milliseconds 450
        return (Test-TrafficLightRunning)
    }
    # Check local target folder
    $localBin = Join-Path $PSScriptRoot "..\target\release\traffic-light.exe"
    if (Test-Path $localBin) {
        Start-Process -FilePath $localBin -WindowStyle Hidden
        Start-Sleep -Milliseconds 450
        return (Test-TrafficLightRunning)
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
        # Silently fail if traffic-light is not running
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
            Ensure-TrafficLightDaemon | Out-Null
            Send-TrafficCommand -Endpoint "/session/on" -Body @{
                session_id = $SessionId
                label = if ($Message) { $Message } else { "Session ($PID)" }
                state = "green"
            }
            Write-Host "[Traffic Light] Session '$SessionId' ON (Green)" -ForegroundColor Green
        }
        "off" {
            Send-TrafficCommand -Endpoint "/session/off" -Body @{
                session_id = $SessionId
            }
            Write-Host "[Traffic Light] Session '$SessionId' OFF" -ForegroundColor DarkGray
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
            Write-Host "[Traffic Light] All sessions cleared." -ForegroundColor DarkYellow
        }
        default {
            Write-Host "Traffic Light CLI Controller" -ForegroundColor Cyan
            Write-Host "User Commands: traffic on | traffic off" -ForegroundColor Green
            Write-Host "Automatic Wrapper: traffic wrap '<command>'" -ForegroundColor Yellow
        }
    }
}

# Short alias /dr
Set-Alias -Name dr -Value traffic -Scope Global
