# SmartOrchestraVST

Plugin VST3 inteligente em Rust usando `nih-plug`.

## Recursos principais

- Detecção automática de velocity para camadas dinâmicas (`pp` a `ff`) com crossfade suave.
- Detecção de duração de nota para `staccato`, `marcato` e `sustain`.
- Detecção de legato por overlap de notas e janela de 30ms entre notas.
- CC1 (modwheel) para dinâmica contínua com smoothing de 5ms.
- CC11 (expression) multiplicando volume final com smoothing de 5ms.
- Síntese interna Saw + Sine, ADSR por articulação, filtro lowpass e até 64 vozes.
- Humanização leve e round robin básico.

## Build com Gradle (pipeline de instalação)

Este projeto agora usa **Gradle** para orquestrar o build (como pedido):

1. Compila plugin e host com Cargo para `x86_64-pc-windows-msvc`
2. Monta `dist/windows-package`
3. Gera instalador `.exe` com Inno Setup

### Pré-requisitos (Windows)

1. Rust + target Windows x64:
   ```powershell
   rustup target add x86_64-pc-windows-msvc
   ```
2. Gradle instalado e disponível no `PATH` (recomendado Java 17 ou 21 em `JAVA_HOME`).
3. Inno Setup 6 (`iscc` ou `ISCC.exe`) no `PATH`.

### Comando principal

```bat
gradle buildWindowsInstaller
```

Ou via atalho:

```bat
build_installer.bat
```

> Se necessário, defina `JAVA_HOME` para JDK 17/21 antes de rodar o Gradle.

### Artefato final

```text
dist/windows-installer/SmartOrchestraVST-Setup-x64.exe
```

### Propriedades opcionais do Gradle

```bat
gradle buildWindowsInstaller -PrustTarget=x86_64-pc-windows-msvc -PrustProfile=release -PcargoCmd=cargo -PisccCmd=iscc
```

## Build Rust padrão (sem instalador)

```bash
cargo build --target x86_64-pc-windows-msvc --release
```

Saídas esperadas:

- Plugin: `target/x86_64-pc-windows-msvc/release/SmartOrchestraVST.vst3`
- Host de teste: `target/x86_64-pc-windows-msvc/release/SmartOrchestraTestHost.exe`

## Test Host (sem DAW)

```bash
cargo run --release --bin SmartOrchestraTestHost -- demo.mid out.wav 48000
```

O host:
- carrega um arquivo MIDI,
- interpreta NoteOn/NoteOff/CC1/CC11,
- renderiza áudio estéreo para WAV.
