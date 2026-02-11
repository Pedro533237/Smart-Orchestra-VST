param(
    [string]$JavaHome = "",
    [string]$GradleCmd = "gradle"
)

$ErrorActionPreference = "Stop"

if ($JavaHome -and (Test-Path $JavaHome)) {
    $env:JAVA_HOME = $JavaHome
}

if (-not $env:JAVA_HOME) {
    Write-Host "JAVA_HOME não definido. O Gradle usará o Java padrão do ambiente."
}

if (-not (Get-Command $GradleCmd -ErrorAction SilentlyContinue)) {
    throw "Gradle não encontrado. Instale o Gradle e tente novamente."
}

Write-Host "==> Executando pipeline Gradle: buildWindowsInstaller"
& $GradleCmd buildWindowsInstaller
