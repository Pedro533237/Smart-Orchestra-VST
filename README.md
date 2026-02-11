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

## Compilação

```bash
cargo build --release
```

Saídas esperadas:

- Plugin: `target/release/SmartOrchestraVST.vst3` (Windows x64 target)
- Host de teste: `target/release/SmartOrchestraTestHost.exe`

## Test Host (sem DAW)

```bash
cargo run --release --bin SmartOrchestraTestHost -- demo.mid out.wav 48000
```

O host:
- carrega um arquivo MIDI,
- interpreta NoteOn/NoteOff/CC1/CC11,
- renderiza áudio estéreo para WAV.
