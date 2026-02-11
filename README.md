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

## Build padrão

```bash
cargo build --release
```

Saídas esperadas no target Windows x64:

- Plugin: `target/x86_64-pc-windows-msvc/release/SmartOrchestraVST.vst3`
- Host de teste: `target/x86_64-pc-windows-msvc/release/SmartOrchestraTestHost.exe`

## Build de instalação (.exe)

Este projeto inclui um pipeline para gerar instalador `.exe` com **Inno Setup 6**.

### Pré-requisitos (Windows)

1. Rust toolchain com target Windows x64:
   ```powershell
   rustup target add x86_64-pc-windows-msvc
   ```
2. Inno Setup 6 instalado (com `ISCC.exe`).

### Gerar instalador

Opção 1 (mais simples):

```bat
build_installer.bat
```

Opção 2 (PowerShell):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_installer.ps1
```

### Saídas do processo de instalação

- Pacote intermediário: `dist/windows-package/`
- Instalador final: `dist/windows-installer/SmartOrchestraVST-Setup-x64.exe`

O instalador copia:
- `SmartOrchestraTestHost.exe` para `Arquivos de Programas\SmartOrchestraVST`
- bundle `SmartOrchestraVST.vst3` para a pasta do app
- opcionalmente, o plugin para `C:\Program Files\Common Files\VST3\SmartOrchestraVST.vst3`

## Test Host (sem DAW)

```bash
cargo run --release --bin SmartOrchestraTestHost -- demo.mid out.wav 48000
```

O host:
- carrega um arquivo MIDI,
- interpreta NoteOn/NoteOff/CC1/CC11,
- renderiza áudio estéreo para WAV.
