# install.ps1 — Script de instalación de ApliArte Link para Windows (PowerShell)
# Uso: irm https://raw.githubusercontent.com/erbolamm/apliarte-link/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$REPO = "erbolamm/apliarte-link"
$BIN_NAME = "apliarte-link"
$INSTALL_DIR = "$env:LOCALAPPDATA\Programs\$BIN_NAME"
$TARGET = "x86_64-pc-windows-msvc"

Write-Host "🔍 Detectando sistema: Windows x64"

# Obtener la última versión disponible
Write-Host "📡 Obteniendo última versión..."
try {
    $release = Invoke-RestMethod "https://api.github.com/repos/$REPO/releases/latest"
    $VERSION = $release.tag_name
} catch {
    Write-Host "⚠️  No se encontró ninguna release publicada."
    Write-Host "   El proyecto está en desarrollo activo."
    Write-Host "   Clona el repo y compila desde código fuente:"
    Write-Host "   git clone https://github.com/$REPO.git apliarte-link"
    Write-Host "   cd apliarte-link"
    Write-Host "   cargo build --release"
    exit 1
}

Write-Host "📦 Versión: $VERSION"

# Descargar el binario .exe
$URL = "https://github.com/$REPO/releases/download/$VERSION/$BIN_NAME-$TARGET.exe"
$TMP = [System.IO.Path]::GetTempPath()
$DEST_EXE = "$TMP\$BIN_NAME.exe"
$INSTALL_EXE = "$INSTALL_DIR\$BIN_NAME.exe"

Write-Host "⬇️  Descargando $BIN_NAME.exe..."
Invoke-WebRequest -Uri $URL -OutFile $DEST_EXE

# Crear directorio de instalación
if (-not (Test-Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Path $INSTALL_DIR | Out-Null
}

# Mover el binario
Move-Item -Force $DEST_EXE $INSTALL_EXE

# Añadir al PATH del usuario si no está ya
$userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$INSTALL_DIR*") {
    Write-Host "🔧 Añadiendo al PATH..."
    [System.Environment]::SetEnvironmentVariable("PATH", "$userPath;$INSTALL_DIR", "User")
    Write-Host "   Reinicia PowerShell para que el PATH se aplique"
}

Write-Host ""
Write-Host "✅ ¡ApliArte Link instalado correctamente!"
Write-Host ""
Write-Host "🚀 Para empezar (reinicia PowerShell primero):"
Write-Host "   apliarte-link          → arranca el servicio"
Write-Host "   apliarte-link --help   → ver todas las opciones"
Write-Host ""
