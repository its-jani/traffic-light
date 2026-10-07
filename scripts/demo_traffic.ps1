# ==============================================================================
# Traffic Status Multi-Session Live Demo (PowerShell)
# Simulates OpenCode and Claude Code running in parallel
# ==============================================================================

. "$PSScriptRoot\traffic.ps1"

Write-Host "🚦 Simulating Multi-Session Agent Traffic Status..." -ForegroundColor Cyan

# 1. Register Session 1 (OpenCode)
Write-Host "`n1. Registering OpenCode session..." -ForegroundColor Yellow
traffic on "OpenCode: feat-auth" "opencode-1"
traffic yellow "Analyzing AST and repository tree..." "opencode-1"
Start-Sleep -Seconds 2

# 2. Register Session 2 (Claude Code)
Write-Host "`n2. Registering Claude Code session..." -ForegroundColor Yellow
traffic on "Claude: test-runner" "claude-2"
traffic yellow "Running test suite (42 tests)..." "claude-2"
Start-Sleep -Seconds 2

# 3. Simulate OpenCode requiring user permission (RED light)
Write-Host "`n3. OpenCode requires terminal permission (RED light)..." -ForegroundColor Red
traffic red "Requires permission to write file 'auth.rs'" "opencode-1"
Start-Sleep -Seconds 2

# 4. Simulate Claude Code finishing successfully (GREEN light)
Write-Host "`n4. Claude Code finished test run (GREEN light)..." -ForegroundColor Green
traffic green "All 42 tests passed" "claude-2"
Start-Sleep -Seconds 2

# 5. User approves OpenCode, transitions back to YELLOW and then GREEN
Write-Host "`n5. OpenCode resumed (YELLOW -> GREEN)..." -ForegroundColor Yellow
traffic yellow "Writing auth.rs and compiling..." "opencode-1"
Start-Sleep -Seconds 2
traffic green "Feature auth completed" "opencode-1"

Write-Host "`n✅ Multi-session simulation complete!" -ForegroundColor Green
