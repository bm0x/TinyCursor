@echo off
setlocal
cd /d "%~dp0"
echo =======================================================
echo   TinyCursor: Compilador & Verificador de Firma
echo =======================================================
powershell.exe -ExecutionPolicy Bypass -NoProfile -File "%~dp0build.ps1"
if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] La compilacion o la firma han fallado.
    pause
    exit /b %ERRORLEVEL%
)
echo.
echo Proceso finalizado correctamente.
pause
