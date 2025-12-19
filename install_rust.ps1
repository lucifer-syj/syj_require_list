# Rust Custom Installation Script - Install to E Drive
# Usage: Run this script in PowerShell

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Rust Custom Installation Script" -ForegroundColor Cyan
Write-Host "Target Directory: E:\DevTools\Rust" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Set environment variables
Write-Host "[1/4] Setting environment variables..." -ForegroundColor Yellow

$rustupHome = "E:\DevTools\Rust\rustup"
$cargoHome = "E:\DevTools\Rust\cargo"
$cargoBin = "$cargoHome\bin"

# Set user environment variables
[System.Environment]::SetEnvironmentVariable('RUSTUP_HOME', $rustupHome, 'User')
[System.Environment]::SetEnvironmentVariable('CARGO_HOME', $cargoHome, 'User')

Write-Host "  RUSTUP_HOME = $rustupHome" -ForegroundColor Green
Write-Host "  CARGO_HOME = $cargoHome" -ForegroundColor Green

# Add to PATH
$currentPath = [System.Environment]::GetEnvironmentVariable('Path', 'User')
if ($currentPath -notlike "*$cargoBin*") {
    [System.Environment]::SetEnvironmentVariable('Path', "$currentPath;$cargoBin", 'User')
    Write-Host "  Added to PATH: $cargoBin" -ForegroundColor Green
} else {
    Write-Host "  Cargo bin already in PATH" -ForegroundColor Green
}

# Refresh environment variables for current session
$env:RUSTUP_HOME = $rustupHome
$env:CARGO_HOME = $cargoHome
$env:Path = "$env:Path;$cargoBin"

Write-Host ""

# Step 2: Create directories
Write-Host "[2/4] Creating installation directories..." -ForegroundColor Yellow

if (-not (Test-Path "E:\DevTools")) {
    New-Item -ItemType Directory -Path "E:\DevTools" -Force | Out-Null
    Write-Host "  Created: E:\DevTools" -ForegroundColor Green
}

if (-not (Test-Path $rustupHome)) {
    New-Item -ItemType Directory -Path $rustupHome -Force | Out-Null
    Write-Host "  Created: $rustupHome" -ForegroundColor Green
}

if (-not (Test-Path $cargoHome)) {
    New-Item -ItemType Directory -Path $cargoHome -Force | Out-Null
    Write-Host "  Created: $cargoHome" -ForegroundColor Green
}

Write-Host ""

# Step 3: Download Rust installer
Write-Host "[3/4] Downloading Rust installer..." -ForegroundColor Yellow

$installerPath = "$env:TEMP\rustup-init.exe"

try {
    Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile $installerPath -UseBasicParsing
    Write-Host "  Download complete: $installerPath" -ForegroundColor Green
} catch {
    Write-Host "  Download failed. Please check your network connection" -ForegroundColor Red
    Write-Host "  Error: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Step 4: Run installer
Write-Host "[4/4] Running Rust installer..." -ForegroundColor Yellow
Write-Host "  Please select default option (type 1 and press Enter)" -ForegroundColor Cyan
Write-Host ""

& $installerPath

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Installation Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Please close this PowerShell window and open a new one, then verify:" -ForegroundColor Yellow
Write-Host "  rustc --version" -ForegroundColor White
Write-Host "  cargo --version" -ForegroundColor White
Write-Host ""
