#!/usr/bin/env pwsh
# Rapter CLI - Build and run Rapter programs
# Usage:
#   rapter build <file.rapt>      - Compile Rapter to executable
#   rapter run                     - Run the last built program
#   rapter compile <file.rapt>     - Just compile to C (output.c)
#   rapter clean                   - Clean build artifacts

param(
    [Parameter(Position = 0)]
    [string]$Command,
    
    [Parameter(Position = 1)]
    [string]$File
)

$ErrorActionPreference = "Stop"

# Configuration
$CARGO_CMD = "cargo"
$RAPTER_DIR = $PSScriptRoot
$BUILD_DIR = Join-Path $RAPTER_DIR ".build"
$LAST_BUILD_FILE = Join-Path $BUILD_DIR "last_build.txt"
$OUTPUT_C = "output.c"
$OUTPUT_EXE = "output.exe"

function Show-Help {
    Write-Host ""
    Write-Host "Rapter CLI - Compile and run Rapter programs" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  rapter build <file.rapt>   - Compile Rapter source to executable"
    Write-Host "  rapter run                  - Run the last built program"
    Write-Host "  rapter compile <file.rapt>  - Compile to C only (output.c)"
    Write-Host "  rapter clean                - Clean build artifacts"
    Write-Host "  rapter help                 - Show this help"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  rapter build hello.rapt"
    Write-Host "  rapter run"
    Write-Host "  rapter build examples/test.rapt"
    Write-Host ""
}

function Ensure-BuildDir {
    if (-not (Test-Path $BUILD_DIR)) {
        New-Item -ItemType Directory -Path $BUILD_DIR | Out-Null
    }
}

function Compile-Rapter {
    param([string]$SourceFile)
    
    if (-not (Test-Path $SourceFile)) {
        Write-Host "Error: File not found: $SourceFile" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "Compiling Rapter: $SourceFile" -ForegroundColor Cyan
    Write-Host ""
    
    # Run cargo to compile Rapter source
    & $CARGO_CMD run --quiet -- $SourceFile
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "Compilation failed!" -ForegroundColor Red
        exit 1
    }
    
    if (-not (Test-Path $OUTPUT_C)) {
        Write-Host ""
        Write-Host "Error: No output.c generated" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "Rapter compilation successful!" -ForegroundColor Green
}

function Build-Executable {
    param([string]$SourceFile)
    
    Ensure-BuildDir
    
    # Compile Rapter source to C
    Compile-Rapter $SourceFile
    
    # Compile C to executable
    Write-Host ""
    Write-Host "Building executable with GCC..." -ForegroundColor Cyan
    
    & gcc $OUTPUT_C -o $OUTPUT_EXE
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "GCC compilation failed!" -ForegroundColor Red
        exit 1
    }
    
    # Save the source file for 'rapter run'
    $SourceFile | Out-File -FilePath $LAST_BUILD_FILE -NoNewline
    
    Write-Host ""
    Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║                                                              ║" -ForegroundColor Green
    Write-Host "║                   BUILD SUCCESSFUL!                          ║" -ForegroundColor Green
    Write-Host "║                                                              ║" -ForegroundColor Green
    Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""
    Write-Host "Executable: $OUTPUT_EXE" -ForegroundColor Cyan
    Write-Host "To run: rapter run" -ForegroundColor Yellow
    Write-Host ""
}

function Run-Program {
    if (-not (Test-Path $OUTPUT_EXE)) {
        Write-Host ""
        Write-Host "Error: No executable found. Build a program first with 'rapter build <file>'" -ForegroundColor Red
        Write-Host ""
        exit 1
    }
    
    Write-Host ""
    Write-Host "Running program..." -ForegroundColor Cyan
    Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor DarkGray
    Write-Host ""
    
    # Run the executable with all remaining arguments
    & ".\$OUTPUT_EXE" $args
    
    $exitCode = $LASTEXITCODE
    
    Write-Host ""
    Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor DarkGray
    if ($exitCode -eq 0) {
        Write-Host "Program exited with code 0" -ForegroundColor Green
    } else {
        Write-Host "Program exited with code $exitCode" -ForegroundColor Yellow
    }
    Write-Host ""
    
    exit $exitCode
}

function Clean-Build {
    Write-Host ""
    Write-Host "Cleaning build artifacts..." -ForegroundColor Cyan
    
    $cleaned = 0
    
    if (Test-Path $OUTPUT_C) {
        Remove-Item $OUTPUT_C
        Write-Host "  Removed: $OUTPUT_C"
        $cleaned++
    }
    
    if (Test-Path $OUTPUT_EXE) {
        Remove-Item $OUTPUT_EXE
        Write-Host "  Removed: $OUTPUT_EXE"
        $cleaned++
    }
    
    if (Test-Path $BUILD_DIR) {
        Remove-Item -Recurse $BUILD_DIR
        Write-Host "  Removed: $BUILD_DIR"
        $cleaned++
    }
    
    Write-Host ""
    if ($cleaned -eq 0) {
        Write-Host "Already clean!" -ForegroundColor Green
    } else {
        Write-Host "Cleaned $cleaned item$(if ($cleaned -ne 1) { 's' })!" -ForegroundColor Green
    }
    Write-Host ""
}

# Main command dispatcher
switch ($Command) {
    "build" {
        if (-not $File) {
            Write-Host ""
            Write-Host "Error: No file specified" -ForegroundColor Red
            Write-Host "Usage: rapter build <file.rapt>" -ForegroundColor Yellow
            Write-Host ""
            exit 1
        }
        Build-Executable $File
    }
    
    "compile" {
        if (-not $File) {
            Write-Host ""
            Write-Host "Error: No file specified" -ForegroundColor Red
            Write-Host "Usage: rapter compile <file.rapt>" -ForegroundColor Yellow
            Write-Host ""
            exit 1
        }
        Compile-Rapter $File
        Write-Host ""
        Write-Host "C code generated: $OUTPUT_C" -ForegroundColor Green
        Write-Host ""
    }
    
    "run" {
        Run-Program @args
    }
    
    "clean" {
        Clean-Build
    }
    
    { $_ -in "help", "-h", "--help", "" } {
        Show-Help
    }
    
    default {
        Write-Host ""
        Write-Host "Error: Unknown command '$Command'" -ForegroundColor Red
        Show-Help
        exit 1
    }
}
