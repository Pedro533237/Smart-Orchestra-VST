@echo off
setlocal

powershell -ExecutionPolicy Bypass -File scripts\build_installer.ps1 %*
if errorlevel 1 (
  echo.
  echo Falha ao gerar instalador.
  exit /b 1
)

echo.
echo Instalador concluido com sucesso.
endlocal
