# TinyCursor - Desinstalador
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-ExecutionPolicy Bypass -NoProfile -File `"$PSCommandPath`""
    exit 0
}

Write-Host "Deteniendo TinyCursor..." -ForegroundColor Cyan
Get-Process -Name "tiny-cursor" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

$installDir = "C:\Program Files\TinyCursor"
if (Test-Path $installDir) {
    Remove-Item -Path $installDir -Recurse -Force
    Write-Host "Carpeta eliminada: $installDir" -ForegroundColor Green
}

$desktopPath = [Environment]::GetFolderPath("Desktop")
$desktopShortcut = Join-Path $desktopPath "TinyCursor.lnk"
if (Test-Path $desktopShortcut) {
    Remove-Item -Path $desktopShortcut -Force
}

$programsPath = [Environment]::GetFolderPath("Programs")
$startShortcut = Join-Path $programsPath "TinyCursor.lnk"
if (Test-Path $startShortcut) {
    Remove-Item -Path $startShortcut -Force
}

Write-Host "TinyCursor ha sido desinstalado correctamente." -ForegroundColor Green
Start-Sleep -Seconds 2
