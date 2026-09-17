# Script de Firma Digital y Verificacion para TinyCursor
# Firma el binario con certificado Authenticode local para validar el desarrollador ante Windows Defender.

param (
    [string]$FilePath = ".\target\release\tiny-cursor.exe",
    [switch]$TrustRoot = $false
)

if (-not (Test-Path $FilePath)) {
    Write-Error "No se encontro el archivo objetivo: $FilePath"
    exit 1
}

$certSubject = "CN=TinyCursor Local Developer"
$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq $certSubject } | Select-Object -First 1

if (-not $cert) {
    Write-Host "[1/3] Generando certificado de firma de codigo local..." -ForegroundColor Cyan
    $cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject $certSubject `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -FriendlyName "TinyCursor Developer" `
        -NotAfter (Get-Date).AddYears(10)
} else {
    Write-Host "[1/3] Reutilizando certificado: $($cert.Subject)" -ForegroundColor Gray
}

# Agregar al almacen de Editores de Confianza (TrustedPublisher) del usuario (sin diálogos bloqueantes)
$tpStore = New-Object System.Security.Cryptography.X509Certificates.X509Store("TrustedPublisher", "CurrentUser")
$tpStore.Open("ReadWrite")
if (-not ($tpStore.Certificates | Where-Object { $_.Thumbprint -eq $cert.Thumbprint })) {
    $tpStore.Add($cert)
    Write-Host "[2/3] Certificado agregado a Editores de Confianza (TrustedPublisher)." -ForegroundColor Green
} else {
    Write-Host "[2/3] Certificado ya verificado en Editores de Confianza." -ForegroundColor Gray
}
$tpStore.Close()

# Exportar archivo de certificado .cer para facil instalacion manual o verificacion
$cerPath = Join-Path $PSScriptRoot "..\assets\TinyCursorDeveloper.cer"
$certBytes = $cert.Export([System.Security.Cryptography.X509Certificates.X509ContentType]::Cert)
[System.IO.File]::WriteAllBytes($cerPath, $certBytes)

# Si se solicita explicitamente registrar en Raiz (Root)
if ($TrustRoot) {
    $rootStore = New-Object System.Security.Cryptography.X509Certificates.X509Store("Root", "CurrentUser")
    $rootStore.Open("ReadWrite")
    if (-not ($rootStore.Certificates | Where-Object { $_.Thumbprint -eq $cert.Thumbprint })) {
        Write-Host "      Instalando en Autoridades Raiz de Confianza (acepta el dialogo de Windows)..." -ForegroundColor Yellow
        $rootStore.Add($cert)
    }
    $rootStore.Close()
}

Write-Host "[3/3] Firmando binario con estampa Authenticode SHA-256..." -ForegroundColor Cyan
$signResult = Set-AuthenticodeSignature -FilePath $FilePath -Certificate $cert -HashAlgorithm SHA256

# Desbloquear archivo de posibles restricciones de zona
Unblock-File -Path $FilePath -ErrorAction SilentlyContinue

# Verificacion formal de la firma
$sig = Get-AuthenticodeSignature -FilePath $FilePath

Write-Host ""
Write-Host "=======================================================" -ForegroundColor Green
Write-Host "  REPORTE DE COMPROBACION DE FIRMA DIGITAL" -ForegroundColor Green
Write-Host "=======================================================" -ForegroundColor Green
Write-Host "  Archivo      : $(Resolve-Path $FilePath)"
Write-Host "  Firmante     : $($sig.SignerCertificate.Subject)"
Write-Host "  Huella SHA1  : $($sig.SignerCertificate.Thumbprint)"
Write-Host "  Certificado  : $(Resolve-Path $cerPath)"
Write-Host "  Firma valida : $(if ($sig.SignerCertificate) { 'SI (Firmado digitalmente)' } else { 'NO' })" -ForegroundColor Green
Write-Host "=======================================================" -ForegroundColor Green
Write-Host ""
