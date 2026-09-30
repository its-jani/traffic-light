# ==============================================================================
# Traffic Light PowerShell Integration Module
# ==============================================================================

$script:TrafficHost = if ($env:TRAFFIC_LIGHT_HOST) { $env:TRAFFIC_LIGHT_HOST } else { "127.0.0.1" }
$script:TrafficPort = if ($env:TRAFFIC_LIGHT_PORT) { $env:TRAFFIC_LIGHT_PORT } else { "8765" }
$script:TrafficBaseUrl = "http://$($script:TrafficHost):$($script:TrafficPort)"

if (-not $env:TRAFFIC_SESSION_ID) {
    $env:TRAFFIC_SESSION_ID = "ps-$PID-$([guid]::NewGuid().ToString().Substring(0,4))"
}

function Send-TrafficLight {
    param(
        [Parameter(Mandatory=$true)] [string]$Endpoint,
        [Parameter(Mandatory=$true)] [hashtable]$Payload
    )
    $json = $Payload | ConvertTo-Json -Compress
    try {
        Invoke-RestMethod -Uri "$($script:TrafficBaseUrl)$Endpoint" `
            -Method Post `
            -ContentType "application/json" `
            -Body $json `
            -TimeoutSec 1 `
            -ErrorAction SilentlyContinue | Out-Null
    } catch {
        # Silent ignore if daemon is not running
    }
}

function Set-TrafficOn {
    param(
        [string]$SessionId = $env:TRAFFIC_SESSION_ID,
        [string]$Label = "PS Session ($PID)"
    )
    Send-TrafficLight -Endpoint "/session/on" -Payload @{
        session_id = $SessionId
        label = $Label
        state = "green"
    }
    Write-Host "[Traffic Light] Session '$SessionId' registered ($Label)." -ForegroundColor Green
}

function Set-TrafficOff {
    param(
        [string]$SessionId = $env:TRAFFIC_SESSION_ID
    )
    Send-TrafficLight -Endpoint "/session/off" -Payload @{
        session_id = $SessionId
    }
    Write-Host "[Traffic Light] Session '$SessionId' dismissed." -ForegroundColor DarkGray
}

function Set-TrafficState {
    param(
        [Parameter(Mandatory=$true)] [string]$State,
        [string]$Message = "",
        [string]$SessionId = $env:TRAFFIC_SESSION_ID
    )
    $payload = @{
        session_id = $SessionId
        state = $State
    }
    if ($Message) {
        $payload["message"] = $Message
    }
    Send-TrafficLight -Endpoint "/state" -Payload $payload
}

function traffic {
    param(
        [string]$Action = "help",
        [string]$Arg1 = "",
        [string]$Arg2 = ""
    )

    switch ($Action.ToLower()) {
        "on" { Set-TrafficOn -SessionId (if ($Arg1) { $Arg1 } else { $env:TRAFFIC_SESSION_ID }) -Label (if ($Arg2) { $Arg2 } else { "PowerShell ($PID)" }) }
        "off" { Set-TrafficOff -SessionId (if ($Arg1) { $Arg1 } else { $env:TRAFFIC_SESSION_ID }) }
        "yellow" { Set-TrafficState -State "yellow" -Message (if ($Arg1) { $Arg1 } else { "Working / Generating..." }) }
        "thinking" { Set-TrafficState -State "yellow" -Message (if ($Arg1) { $Arg1 } else { "Thinking..." }) }
        "working" { Set-TrafficState -State "yellow" -Message (if ($Arg1) { $Arg1 } else { "Working..." }) }
        "green" { Set-TrafficState -State "green" -Message (if ($Arg1) { $Arg1 } else { "Ready for next prompt" }) }
        "idle" { Set-TrafficState -State "green" -Message (if ($Arg1) { $Arg1 } else { "Idle" }) }
        "done" { Set-TrafficState -State "green" -Message (if ($Arg1) { $Arg1 } else { "Completed" }) }
        "red" { Set-TrafficState -State "red" -Message (if ($Arg1) { $Arg1 } else { "Needs user input / Halted" }) }
        "input" { Set-TrafficState -State "red" -Message (if ($Arg1) { $Arg1 } else { "Waiting for user input" }) }
        "error" { Set-TrafficState -State "red" -Message (if ($Arg1) { $Arg1 } else { "Error / Halted" }) }
        "clear" { Send-TrafficLight -Endpoint "/clear" -Payload @{} }
        default {
            Write-Host "Traffic Light CLI (PowerShell)" -ForegroundColor Cyan
            Write-Host "Usage: traffic <on|off|yellow|green|red|clear> [message/id]"
            Write-Host "Examples:"
            Write-Host "  traffic on                        - Register current terminal"
            Write-Host "  traffic yellow 'Generating code'  - Set yellow light"
            Write-Host "  traffic green 'Task complete'     - Set green light"
            Write-Host "  traffic red 'Waiting for input'   - Set red light"
            Write-Host "  traffic off                       - Dismiss session"
        }
    }
}
