# =====================================================================
# TinyCursor - Pipeline Completo de Compilacion, Firma y Verificacion
# =====================================================================
# Este script compila la version Release optimizada en Rust,
# genera/aplica la firma digital Authenticode de desarrollador local
# y verifica el estado de comprobacion para Windows Defender.
# =====================================================================

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "`n=======================================================" -ForegroundColor Cyan
Write-Host "   TinyCursor: Compilador & Verificador de Firma" -ForegroundColor Cyan
Write-Host "=======================================================`n" -ForegroundColor Cyan

# 1. Detener instancias previas en ejecucion
Get-Process -Name "tiny-cursor" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 2. Localizar Cargo / Rust
$cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
if (-not (Test-Path $cargoPath)) {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        $cargoPath = "cargo"
    } else {
        Write-Error "No se encontro Cargo en el sistema. Verifica que Rust este instalado."
        exit 1
    }
}

# 2. Ejecutar Pruebas Unitarias
Write-Host "[Paso 1/3] Ejecutando pruebas unitarias de fisicas y vectores..." -ForegroundColor Cyan
& $cargoPath test --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Error "Fallaron las pruebas unitarias. Abortando compilacion."
    exit $LASTEXITCODE
}
Write-Host "  -> Todas las pruebas unitarias pasaron con exito.`n" -ForegroundColor Green

# 3. Compilar en Modo Release con Manifest UIAccess y Linker rust-lld
Write-Host "[Paso 2/3] Compilando binario Release optimizado en Rust con UIAccess..." -ForegroundColor Cyan

# Asegurar manifest.res
$makeResScript = Join-Path $PSScriptRoot "tools\make_res.py"
if (Test-Path $makeResScript) {
    & python $makeResScript | Out-Null
}

$exePath = Join-Path $PSScriptRoot "target\release\tiny-cursor.exe"
Remove-Item -Path $exePath -Force -ErrorAction SilentlyContinue

& $cargoPath rustc --release -- -C linker=rust-lld -C link-arg="manifest.res"
if ($LASTEXITCODE -ne 0) {
    Write-Error "Error durante la compilacion de Cargo."
    exit $LASTEXITCODE
}
Write-Host "  -> Binario compilado exitosamente en target\release\tiny-cursor.exe.`n" -ForegroundColor Green

# 4. Firma Digital y Verificacion de Comprobaciones
Write-Host "[Paso 3/3] Aplicando firma digital Authenticode y verificando..." -ForegroundColor Cyan
$signScript = Join-Path $PSScriptRoot "tools\sign.ps1"
$exePath = Join-Path $PSScriptRoot "target\release\tiny-cursor.exe"

& powershell.exe -ExecutionPolicy Bypass -File $signScript -FilePath $exePath

Write-Host "Compilacion y firma finalizadas. El ejecutable esta listo para usarse:" -ForegroundColor Green
Write-Host "  $exePath`n" -ForegroundColor White
