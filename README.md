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

## What still needs to be verified on Windows

This was all written without the ability to run it, so treat first runs as
a debugging pass, not just a demo:

- `npm run tauri dev` actually launches and the UI renders correctly.
- Real microphone/speaker enumeration and capture via CPAL's WASAPI backend.
- Sample-rate/format negotiation between your device's native format and the
  24kHz mono PCM16 OpenAI's Realtime API expects — listen for aliasing or
  glitches from the linear-interpolation resampler in `src-tauri/src/audio.rs`.
- Whether the "translate, don't converse" system prompt in
  `src-tauri/src/websocket.rs` reliably produces translations rather than
  the model answering questions or chatting back — likely needs prompt
  iteration.
- Reconnect/backoff behavior if the connection drops mid-session.
- That `settings.json` (stored under your Windows `AppData` config dir)
  round-trips correctly across app restarts.
- The exact OpenAI Realtime model id, beta header, and audio format in
  `src-tauri/src/websocket.rs` — these can drift from OpenAI's docs after
  this code was written; check https://platform.openai.com/docs/guides/realtime
  if the connection fails immediately.

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
