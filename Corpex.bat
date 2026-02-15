@echo off
title Corpex
color 0F

:: Add cargo to PATH
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

cd /d "%~dp0"

echo.
echo   Building Corpex...
echo.

cargo build --release --quiet 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo   Build failed. Running again with details:
    echo.
    cargo build --release 2>&1
    echo.
    pause
    exit /b 1
)

echo.
echo   Build complete. Launching Corpex...
echo.
start "" "target\release\corpex.exe"
