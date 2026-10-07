# ==============================================================================
# Traffic Status - Standalone Installer for Windows (PowerShell)
# Usage: irm https://github.com/its-jani/traffic-status/releases/latest/download/install.ps1 | iex
# ==============================================================================
$ErrorActionPreference = "Stop"

$Repo = "its-jani/traffic-status"
$Target = "x86_64-pc-windows-msvc"
$ArchiveName = "traffic-status-$Target.zip"
$BaseUrl = "https://github.com/$Repo/releases/latest/download"

Write-Host "🚦 Installing Traffic Status..." -ForegroundColor Cyan

# Determine destination directory
$BinDir = Join-Path $env:LOCALAPPDATA "traffic-status\bin"
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

try {
    # 1. Download archive & SHA256SUMS
    $ZipPath = Join-Path $TempDir $ArchiveName
    $SumsPath = Join-Path $TempDir "SHA256SUMS"

    Write-Host "📥 Downloading $ArchiveName..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "$BaseUrl/$ArchiveName" -OutFile $ZipPath -UseBasicParsing
    Invoke-WebRequest -Uri "$BaseUrl/SHA256SUMS" -OutFile $SumsPath -UseBasicParsing

    # 2. Verify SHA256 Checksum
    Write-Host "🔒 Verifying SHA-256 checksum..." -ForegroundColor Yellow
    $ComputedHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()
    $ExpectedLine = (Get-Content $SumsPath | Where-Object { $_ -match $ArchiveName })
    
    if (-not $ExpectedLine) {
        throw "Could not find checksum for $ArchiveName in SHA256SUMS"
    }

    $ExpectedHash = ($ExpectedLine -split '\s+')[0].ToLower()
    if ($ComputedHash -ne $ExpectedHash) {
        throw "Checksum verification failed! Expected $ExpectedHash, computed $ComputedHash"
    }
    Write-Host "  ✅ Checksum verified." -ForegroundColor Green

    # 3. Extract binary
    Write-Host "📦 Extracting binary..." -ForegroundColor Yellow
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force
    $ExtractedExe = (Get-ChildItem -Path $TempDir -Recurse -Filter "traffic-status.exe" | Select-Object -First 1).FullName
    
    if (-not $ExtractedExe) {
        throw "Failed to find traffic-status.exe in extracted archive"
    }

    $DestExe = Join-Path $BinDir "traffic-status.exe"
    Copy-Item -Path $ExtractedExe -Destination $DestExe -Force
    Write-Host "  ✅ Installed to: $DestExe" -ForegroundColor Green

    # 4. Add to User PATH if not already present
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($UserPath -notlike "*$BinDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$BinDir", "User")
        $env:PATH = "$env:PATH;$BinDir"
        Write-Host "  ✅ Added $BinDir to User PATH." -ForegroundColor Green
    }

    # 5. Run global hooks installer
    Write-Host "`n⚙️ Running initial global configuration..." -ForegroundColor Yellow
    & "$DestExe" install --global

} finally {
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "`n🎉 Traffic Status installation complete!" -ForegroundColor Green
Write-Host "Run 'traffic-status doctor' to verify system health." -ForegroundColor Cyan
