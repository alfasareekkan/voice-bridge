# VoxBridge

Real-time AI voice translation for Google Meet (English ↔ Malayalam), built
with Tauri v2 + React/TypeScript + Rust. See `VoxBridge_MVP_Specification.md`
for the full product spec.

**Current scope**: the "First Milestone" — capture microphone audio, stream
it to OpenAI's Realtime API, and play the translated speech back through
speakers. Virtual-microphone output (so Google Meet picks up the translated
audio directly) is a stubbed-out Phase 2 extension point (see
`src-tauri/src/virtual_mic.rs`), not yet implemented.

This project was scaffolded and coded inside WSL2 Linux, where Tauri's
webview and real audio devices aren't available. Everything here compiles
(`cargo check` / `npm run build`), but running and testing it requires a
native Windows machine.

## Prerequisites (Windows)

1. **Rust** — install via [rustup](https://www.rust-lang.org/tools/install)
   with the MSVC toolchain (the default on Windows).
2. **WebView2 runtime** — usually already present on Windows 10/11; if not,
   [download it here](https://developer.microsoft.com/microsoft-edge/webview2/).
3. **Node.js** (v18+) and npm.
4. **An OpenAI API key** with access to the Realtime API.
5. *(Later, for the virtual-mic phase)* a virtual audio cable driver such as
   [VB-CABLE](https://vb-audio.com/Cable/), so Google Meet can pick a virtual
   mic as its input device.

## Prerequisites (macOS)

1. **Rust** — install via [rustup](https://www.rust-lang.org/tools/install).
2. **Xcode Command Line Tools** — `xcode-select --install`.
3. **Node.js** (v18+) and npm.
4. **An OpenAI API key** with access to the Realtime API.
5. *(Later, for the virtual-mic phase)* a virtual audio cable such as [BlackHole](https://existential.audio/blackhole/).

## Setup

```powershell
git clone <this repo>
cd voice-convertor
npm install
npm run tauri dev
```

The first `npm run tauri dev` will take a while as Cargo compiles all Rust
dependencies (cpal, tokio, tokio-tungstenite, etc.).

## Using the app

1. Open **Settings** (gear icon) and paste your OpenAI API key, then **Save Key**.
2. Pick your input microphone and output speaker from the dropdowns.
3. Choose the language pair (defaults to Malayalam → English; use the swap
   button to flip direction).
4. Click **Start**. Status should move to "Connecting…" then "Connected".
5. Speak into the microphone — the translated transcript should appear, and
   translated speech should play through the selected speaker.
6. Click **Stop** to end the session.

**Use headphones while testing** — without them, the speaker output can
feed back into the microphone.

## Running inside WSL2: empty device dropdowns

If you run `npm run tauri dev` inside WSL2 (as this project was scaffolded),
the input/output microphone dropdowns will show **"No devices found"** and
appear unusable. This is not an application bug: the Rust backend enumerates
devices via `cpal`'s plain ALSA backend, and WSL2 exposes no real ALSA
hardware by default, so an empty device list is the correct behavior for
that environment.

WSLg does provide a working PulseAudio bridge to the Windows host's real
microphone/speakers (check with `pactl info` — socket at
`/mnt/wslg/PulseServer` — and `pactl list short sources/sinks`, which shows
`RDPSource`/`RDPSink`). ALSA just isn't routed through it by default. To fix
this for local dev, install the ALSA→PulseAudio bridge and route ALSA's
default device through it:

```bash
sudo apt install libasound2-plugins pulseaudio-utils alsa-utils
```

Then create `~/.asoundrc` with:

```
pcm.pulse {
    type pulse
}
ctl.pulse {
    type pulse
}

pcm.!default {
    type pulse
}
ctl.!default {
    type pulse
}
```

`~/.asoundrc` is a personal, per-machine dev environment file — it's not
managed by this repo and shouldn't be committed. After creating it, `cpal`
should enumerate the bridged host mic/speaker and the dropdowns should
populate.

This is a best-effort workaround for local dev convenience only. Real
device enumeration and audio verification should still ultimately happen on
actual Windows, the app's real target platform — see the next section.

## What still needs to be verified on Windows and macOS

This was all written without the ability to run it, so treat first runs as
a debugging pass, not just a demo:

- `npm run tauri dev` actually launches and the UI renders correctly.
- Real microphone/speaker enumeration and capture via CPAL's WASAPI backend.
- Sample-rate/format negotiation between your device's native format and the
  24kHz mono PCM16 OpenAI's Realtime API expects — listen for aliasing or
  glitches from the linear-interpolation resampler in `src-tauri/src/audio.rs`.
- Whether the GA Realtime Translations endpoint in
  `src-tauri/src/websocket.rs` actually accepts the session-config shape and
  headers this code sends — the model/endpoint/session-config details were
  not confirmed against a live connection when written.
- Reconnect/backoff behavior if the connection drops mid-session.
- That `settings.json` (stored under your Windows `AppData` config dir)
  round-trips correctly across app restarts.
- The exact OpenAI Realtime Translations model id, headers, and audio format
  in `src-tauri/src/websocket.rs` — these can drift from OpenAI's docs after
  this code was written; check
  https://developers.openai.com/api/docs/guides/realtime-translation if the
  connection fails immediately.

## Project structure

- `src/` — React frontend (thin control/display surface; never touches raw
  audio or the API key).
- `src-tauri/src/` — Rust backend, where all the real work happens:
  - `audio.rs` — CPAL device enumeration, resampling, playback stream.
  - `microphone.rs` — microphone capture.
  - `websocket.rs` — OpenAI Realtime API WebSocket client.
  - `session.rs` — orchestrates capture → WebSocket → playback, with
    reconnect logic.
  - `settings.rs` — local settings/API key persistence.
  - `virtual_mic.rs` — Phase 2 stub for virtual microphone output.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
