@echo off
setlocal

if "%JAVA_HOME%"=="" (
  if exist "%USERPROFILE%\.local\share\mise\installs\java\21\bin\java.exe" (
    set "JAVA_HOME=%USERPROFILE%\.local\share\mise\installs\java\21"
  )
)

where gradle >nul 2>nul
if errorlevel 1 (
  echo Gradle nao encontrado no PATH.
  echo Instale o Gradle e rode: gradle buildWindowsInstaller
  exit /b 1
)

gradle buildWindowsInstaller %*
if errorlevel 1 (
  echo.
  echo Falha ao gerar instalador com Gradle.
  exit /b 1
)

echo.
echo Instalador concluido com sucesso.
endlocal
