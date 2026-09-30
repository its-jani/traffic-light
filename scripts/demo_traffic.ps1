# ==============================================================================
# Traffic Light Multi-Session Live Demo (PowerShell)
# Simulates OpenCode, Claude Code, and Gemini CLI running in parallel
# ==============================================================================

. "$PSScriptRoot\traffic.ps1"

Write-Host "🚦 Simulating Multi-Session Agent Traffic Lights..." -ForegroundColor Cyan

# 1. Register Session 1 (OpenCode)
Write-Host "`n1. Registering OpenCode session..." -ForegroundColor Yellow
traffic on "opencode-1" "OpenCode: feat-auth"
traffic yellow "Analyzing AST and repository tree..."
Start-Sleep -Seconds 2

# 2. Register Session 2 (Claude Code)
Write-Host "`n2. Registering Claude Code session..." -ForegroundColor Yellow
traffic on "claude-2" "Claude Code: test-runner"
Set-TrafficState -SessionId "claude-2" -State "yellow" -Message "Running test suite (42 tests)..."
Start-Sleep -Seconds 2

# 3. Simulate OpenCode requiring user permission (RED light)
Write-Host "`n3. OpenCode requires terminal permission (RED light)..." -ForegroundColor Red
Set-TrafficState -SessionId "opencode-1" -State "red" -Message "Requires permission to write file 'auth.rs'"
Start-Sleep -Seconds 2

# 4. Simulate Claude Code finishing successfully (GREEN light)
Write-Host "`n4. Claude Code finished test run (GREEN light)..." -ForegroundColor Green
Set-TrafficState -SessionId "claude-2" -State "green" -Message "All 42 tests passed"
Start-Sleep -Seconds 2

# 5. User approves OpenCode, transitions back to YELLOW and then GREEN
Write-Host "`n5. OpenCode resumed (YELLOW -> GREEN)..." -ForegroundColor Yellow
Set-TrafficState -SessionId "opencode-1" -State "yellow" -Message "Writing auth.rs and compiling..."
Start-Sleep -Seconds 2
Set-TrafficState -SessionId "opencode-1" -State "green" -Message "Feature auth completed"

Write-Host "`n✅ Multi-session simulation complete!" -ForegroundColor Green
