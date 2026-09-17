# Deploy-TinyCursor.ps1
# Requires Administrator privileges to install trusted root certificate and deploy to Program Files
param(
    [string]$TargetBinary = "..\target\release\tiny-cursor.exe"
)

$certName = "TinyCursorLocalDevCert"
$installFolder = "$env:ProgramFiles\TinyCursor"
$destExe = "$installFolder\TinyCursor.exe"

Write-Host "[1/4] Creando o verificando certificado de firma local..." -ForegroundColor Cyan
$cert = Get-ChildItem Cert:\LocalMachine\My | Where-Object { $_.Subject -match $certName }
if (-not $cert) {
    $cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject "CN=$certName" `
        -CertStoreLocation "Cert:\LocalMachine\My" `
        -KeyUsage DigitalSignature `
        -FriendlyName "TinyCursor Code Signing Cert" `
        -NotAfter (Get-Date).AddYears(5)
}

# Anadir a entidades de certificacion raiz de confianza
$rootStore = Get-Item "Cert:\LocalMachine\Root"
$rootStore.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
if (-not ($rootStore.Certificates | Where-Object { $_.Thumbprint -eq $cert.Thumbprint })) {
    $rootStore.Add($cert)
}
$rootStore.Close()

Write-Host "[2/4] Buscando signtool.exe en Windows SDK..." -ForegroundColor Cyan
$signtool = (Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue | Sort-Object FullName -Descending | Select-Object -First 1).FullName

if (-not $signtool) {
    Write-Warning "signtool.exe no encontrado en Program Files (x86). Verificando PATH..."
    $signtool = (Get-Command signtool.exe -ErrorAction SilentlyContinue).Source
}

if ($signtool) {
    Write-Host "[3/4] Firmando binario ($TargetBinary) para habilitar UIAccess..." -ForegroundColor Cyan
    & $signtool sign /fd SHA256 /sha1 $($cert.Thumbprint) /v $TargetBinary
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Error firmando el ejecutable."
        exit 1
    }
} else {
    Write-Warning "signtool.exe no disponible en este equipo. Omitiendo paso de firma."
}

Write-Host "[4/4] Desplegando en directorio protegido ($installFolder)..." -ForegroundColor Cyan
if (-not (Test-Path $installFolder)) {
    New-Item -ItemType Directory -Path $installFolder -Force | Out-Null
}

Stop-Process -Name "TinyCursor" -ErrorAction SilentlyContinue
Stop-Process -Name "tiny-cursor" -ErrorAction SilentlyContinue
Copy-Item -Path $TargetBinary -Destination $destExe -Force

Write-Host "Despliegue finalizado. Iniciando TinyCursor en capa superior (ZBID_UIACCESS)..." -ForegroundColor Green
Start-Process -FilePath $destExe
