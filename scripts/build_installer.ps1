param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

function Ensure-Command([string]$name) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        throw "Comando '$name' não encontrado no PATH."
    }
}

Write-Host "==> Build do plugin e host ($Target / $Profile)"
Ensure-Command cargo
cargo build --target $Target --profile $Profile

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$targetDir = Join-Path $repoRoot "target/$Target/$Profile"
$distDir = Join-Path $repoRoot "dist/windows-package"
$installerOutDir = Join-Path $repoRoot "dist/windows-installer"

if (Test-Path $distDir) {
    Remove-Item -Recurse -Force $distDir
}
New-Item -ItemType Directory -Force -Path $distDir | Out-Null
New-Item -ItemType Directory -Force -Path $installerOutDir | Out-Null

$hostExe = Join-Path $targetDir "SmartOrchestraTestHost.exe"
if (-not (Test-Path $hostExe)) {
    throw "Host de teste não encontrado: $hostExe"
}
Copy-Item $hostExe (Join-Path $distDir "SmartOrchestraTestHost.exe") -Force

$vst3Bundle = Join-Path $targetDir "SmartOrchestraVST.vst3"
if (-not (Test-Path $vst3Bundle)) {
    throw "Bundle VST3 não encontrado: $vst3Bundle"
}
Copy-Item $vst3Bundle (Join-Path $distDir "SmartOrchestraVST.vst3") -Recurse -Force

$issScript = Join-Path $repoRoot "installer/SmartOrchestraVST.iss"

$innoCmd = $null
if (Get-Command iscc -ErrorAction SilentlyContinue) {
    $innoCmd = "iscc"
} elseif (Test-Path "C:\Program Files (x86)\Inno Setup 6\ISCC.exe") {
    $innoCmd = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
}

if (-not $innoCmd) {
    throw "Inno Setup (ISCC.exe) não encontrado. Instale o Inno Setup 6 e rode novamente."
}

Write-Host "==> Gerando instalador .exe"
& $innoCmd $issScript | Out-Host

Write-Host "==> Instalador gerado em: $installerOutDir"
Get-ChildItem $installerOutDir -Filter "*.exe" | ForEach-Object {
    Write-Host " - $($_.FullName)"
}
