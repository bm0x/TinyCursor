@echo off
setlocal
cd /d "%~dp0"
title TinyCursor - Instalador UIAccess

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo =======================================================
    echo    TinyCursor - Instalador con Privilegio UIAccess
    echo =======================================================
    echo Solicitando permisos de Administrador de Windows...
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Start-Process cmd -ArgumentList '/c cd /d \"%~dp0\" && call \"%~f0\"' -Verb RunAs"
    exit /b
)

echo =======================================================
echo    TinyCursor - Instalador con Privilegio UIAccess
echo =======================================================
echo.
powershell.exe -ExecutionPolicy Bypass -NoProfile -File "%~dp0install.ps1"
if %ERRORLEVEL% neq 0 (
    echo.
    echo Ocurrio un error durante la ejecucion del instalador.
    pause
)
