#!/usr/bin/env pwsh
# Rapter Language Installer
# Adds Rapter to your PATH so you can use 'rapter' from anywhere

$ErrorActionPreference = "Stop"

$RAPTER_DIR = $PSScriptRoot

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Rapter Language Installer" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if already in PATH
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -like "*$RAPTER_DIR*") {
    Write-Host "Rapter is already in your PATH!" -ForegroundColor Green
    Write-Host "Location: $RAPTER_DIR" -ForegroundColor Gray
    Write-Host ""
    Write-Host "You can use 'rapter' from any directory." -ForegroundColor Green
    Write-Host ""
    exit 0
}

Write-Host "This will add Rapter to your PATH environment variable." -ForegroundColor Yellow
Write-Host ""
Write-Host "Installation directory: $RAPTER_DIR" -ForegroundColor Gray
Write-Host ""
Write-Host "After installation, you can use 'rapter' from any directory." -ForegroundColor Gray
Write-Host ""

# Ask for confirmation
$response = Read-Host "Continue? (Y/n)"
if ($response -and $response -ne "Y" -and $response -ne "y") {
    Write-Host ""
    Write-Host "Installation cancelled." -ForegroundColor Yellow
    Write-Host ""
    exit 0
}

Write-Host ""
Write-Host "Adding Rapter to PATH..." -ForegroundColor Cyan

try {
    # Get current user PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    
    # Add Rapter directory
    if ($userPath) {
        $newPath = "$userPath;$RAPTER_DIR"
    } else {
        $newPath = $RAPTER_DIR
    }
    
    # Set the new PATH
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    
    # Update current session
    $env:Path += ";$RAPTER_DIR"
    
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "  Installation Successful!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Rapter has been added to your PATH." -ForegroundColor Green
    Write-Host ""
    Write-Host "IMPORTANT: You need to restart your terminal for this to take effect" -ForegroundColor Yellow
    Write-Host "in new windows. This terminal session has been updated." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Try it now:" -ForegroundColor Cyan
    Write-Host "  rapter build examples/hello_cli.rapt" -ForegroundColor Gray
    Write-Host "  rapter run" -ForegroundColor Gray
    Write-Host ""
    
} catch {
    Write-Host ""
    Write-Host "Installation failed: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "You may need to run PowerShell as Administrator." -ForegroundColor Yellow
    Write-Host ""
    exit 1
}
