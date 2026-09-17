# =====================================================================
# TinyCursor - Instalador y Activador de UIAccess (Capa Superior Windows)
# =====================================================================
# Este script instala TinyCursor en C:\Program Files\TinyCursor,
# registra el certificado en los almacenes del equipo local y usuario,
# y permite que TinyCursor flote de manera nativa por encima del Menú Inicio de Windows 11,
# el Centro de Notificaciones y el Administrador de Tareas (Banda ZBID_UIACCESS).
# =====================================================================

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

$sourceExe = Join-Path $PSScriptRoot "target\release\tiny-cursor.exe"
$certFile = Join-Path $PSScriptRoot "assets\TinyCursorDeveloper.cer"

# Si no está compilado, compilarlo primero en la sesión del usuario (donde rustc y cargo están disponibles)
if (-not (Test-Path $sourceExe)) {
    Write-Host "`n[Preparacion] Compilando y firmando version Release de TinyCursor..." -ForegroundColor Cyan
    & powershell.exe -ExecutionPolicy Bypass -NoProfile -File (Join-Path $PSScriptRoot "build.ps1")
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $sourceExe)) {
        Write-Error "Fallo la compilacion inicial. Verifica que Rust este instalado."
        Read-Host "Presiona Enter para salir..."
        exit 1
    }
}

# 1. Comprobar elevacion (Administrador requerido para instalar en Program Files y Root Store)
if (-not $isAdmin) {
    Write-Host "`nTinyCursor requiere privilegios de Administrador para instalarse en Program Files y activar la banda UIAccess." -ForegroundColor Yellow
    Write-Host "Solicitando confirmacion de permisos de Administrador..." -ForegroundColor Yellow
    $scriptPath = $PSCommandPath
    try {
        Start-Process powershell.exe -Verb RunAs -ArgumentList "-ExecutionPolicy Bypass -NoProfile -File `"$scriptPath`""
    } catch {
        Write-Warning "No se pudo invocar el dialogo UAC directamente: $_"
        Write-Host "`nPor favor ejecuta 'install.bat' haciendo clic derecho -> 'Ejecutar como administrador'." -ForegroundColor Yellow
        Read-Host "Presiona Enter para salir..."
    }
    exit 0
}

Write-Host "`n=======================================================" -ForegroundColor Cyan
Write-Host "   Instalador de TinyCursor (Nivel UIAccess & Banda DWM)" -ForegroundColor Cyan
Write-Host "=======================================================`n" -ForegroundColor Cyan

# 2. Detener instancias activas de tiny-cursor
Write-Host "[1/5] Deteniendo instancias previas de TinyCursor..." -ForegroundColor Cyan
Get-Process -Name "tiny-cursor" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 400

# 3. Verificar binario Release
Write-Host "[2/5] Verificando binario compilado..." -ForegroundColor Cyan
if (-not (Test-Path $sourceExe)) {
    Write-Host "Compilando binario..." -ForegroundColor Yellow
    & powershell.exe -ExecutionPolicy Bypass -NoProfile -File (Join-Path $PSScriptRoot "build.ps1")
}
Write-Host "  -> Binario verificado: $sourceExe" -ForegroundColor Green

# 4. Registrar Certificado en Almacenes del Equipo (LocalMachine y CurrentUser)
Write-Host "[3/5] Verificando certificado de firma en almacenes de confianza..." -ForegroundColor Cyan

if (-not (Test-Path $certFile)) {
    & powershell.exe -ExecutionPolicy Bypass -NoProfile -File (Join-Path $PSScriptRoot "tools\sign.ps1") -FilePath $sourceExe
}

$cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($certFile)

foreach ($storeLocation in @("LocalMachine", "CurrentUser")) {
    foreach ($storeName in @("Root", "TrustedPublisher")) {
        try {
            $store = New-Object System.Security.Cryptography.X509Certificates.X509Store($storeName, $storeLocation)
            $store.Open("ReadWrite")
            if (-not ($store.Certificates | Where-Object { $_.Thumbprint -eq $cert.Thumbprint })) {
                $store.Add($cert)
                Write-Host "  -> Certificado registrado en $storeLocation\$storeName." -ForegroundColor Green
            } else {
                Write-Host "  -> Certificado ya presente en $storeLocation\$storeName." -ForegroundColor Gray
            }
            $store.Close()
        } catch {
            Write-Warning "No se pudo registrar en $storeLocation\${storeName}: $_"
        }
    }
}

# 5. Instalar archivos en C:\Program Files\TinyCursor
$installDir = "C:\Program Files\TinyCursor"
Write-Host "[4/5] Desplegando en carpeta segura del sistema ($installDir)..." -ForegroundColor Cyan

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$destExe = Join-Path $installDir "tiny-cursor.exe"
Copy-Item -Path $sourceExe -Destination $destExe -Force
Unblock-File -Path $destExe -ErrorAction SilentlyContinue

$destAssets = Join-Path $installDir "assets"
if (-not (Test-Path $destAssets)) {
    New-Item -ItemType Directory -Path $destAssets -Force | Out-Null
}
Copy-Item -Path (Join-Path $PSScriptRoot "assets\*") -Destination $destAssets -Force

Write-Host "  -> Archivos copiados y desbloqueados correctamente." -ForegroundColor Green

# 6. Crear Accesos Directos (Escritorio y Menú Inicio)
$wshShell = New-Object -ComObject WScript.Shell

# Acceso directo Escritorio
$desktopPath = [Environment]::GetFolderPath("Desktop")
$desktopShortcut = $wshShell.CreateShortcut((Join-Path $desktopPath "TinyCursor.lnk"))
$desktopShortcut.TargetPath = $destExe
$desktopShortcut.WorkingDirectory = $installDir
$desktopShortcut.Description = "TinyCursor Smooth Cursor"
if (Test-Path (Join-Path $destAssets "app_icon.ico")) {
    $desktopShortcut.IconLocation = (Join-Path $destAssets "app_icon.ico")
}
$desktopShortcut.Save()

# Acceso directo Menú Inicio
$programsPath = [Environment]::GetFolderPath("Programs")
$startMenuShortcut = $wshShell.CreateShortcut((Join-Path $programsPath "TinyCursor.lnk"))
$startMenuShortcut.TargetPath = $destExe
$startMenuShortcut.WorkingDirectory = $installDir
$startMenuShortcut.Description = "TinyCursor Smooth Cursor"
if (Test-Path (Join-Path $destAssets "app_icon.ico")) {
    $startMenuShortcut.IconLocation = (Join-Path $destAssets "app_icon.ico")
}
$startMenuShortcut.Save()
Write-Host "  -> Accesos directos creados en Escritorio y Menú Inicio." -ForegroundColor Green

# 7. Iniciar TinyCursor desde Program Files
Write-Host "[5/5] Iniciando TinyCursor con privilegio UIAccess activado..." -ForegroundColor Cyan
try {
    Start-Process -FilePath $destExe -WorkingDirectory $installDir
    Write-Host "  -> TinyCursor iniciado exitosamente en segundo plano." -ForegroundColor Green
} catch {
    Write-Warning "Fallo al iniciar el proceso: $_"
}

Write-Host "`n=======================================================" -ForegroundColor Green
Write-Host "   INSTALACION Y CONFIGURACION COMPLETADA" -ForegroundColor Green
Write-Host "=======================================================" -ForegroundColor Green
Write-Host "  Ruta Instalada : $destExe"
Write-Host "  Nivel DWM      : Banda ZBID_UIACCESS (2)"
Write-Host "  Capas Superadas: Menu Inicio Windows 11, Notificaciones y Task Manager"
Write-Host "  Estado         : Ejecutandose en segundo plano (Bandeja del Sistema)"
Write-Host "=======================================================`n"

Write-Host "Presiona Enter para finalizar..." -ForegroundColor Gray
$null = Read-Host
