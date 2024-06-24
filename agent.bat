@echo off
echo Choise system type to launch:
echo 1. Unix
echo 2. Windows
set /p choice=(1 or 2):
echo.

if "%choice%"=="1" (
    echo launch version for Unix...
    cargo run --bin amunix
) else if "%choice%"=="2" (
    echo launch version for Windows...
    cargo run --bin amwin
) else (
    echo err please choice 1 or 2.
)

pause
