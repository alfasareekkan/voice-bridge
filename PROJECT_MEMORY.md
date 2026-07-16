# VoxBridge — Project Memory

A model-agnostic reference to the codebase: what exists, why, and how the
pieces fit together. Written so any AI assistant (or human) can pick up this
project cold, without re-reading every source file first. If code and this
document disagree, trust the code and update this file.

Full product vision/spec: `VoxBridge_MVP_Specification.md`. Read that first
for *why* this product exists; this file is about *what has been built*.

---

## 1. What this is

VoxBridge is a Tauri v2 + React/TypeScript + Rust desktop app for real-time
AI voice translation. A user speaks into their microphone, speech is
streamed to OpenAI's Realtime API, and the translated speech is played back
through speakers (eventually also through a virtual microphone so apps like
Google Meet pick it up as mic input).

**Current build scope ("First Milestone", per the spec's own recommended
sequencing):** mic capture → OpenAI Realtime API → speaker playback. No
virtual microphone, no Google Meet integration yet — those are stubbed
extension points for Phase 2. Full MVP scope (English ↔ Malayalam only,
Windows only, no auth/billing/voice cloning) is defined in the spec.

## 2. Build status (as of last verification)

- **Frontend**: `npm run build` and `npx tsc --noEmit` both pass cleanly.
- **Backend**: `cargo check` in `src-tauri/` has **not yet succeeded**. It
  is blocked on missing Linux system packages (`pkg-config`,
  `libdbus-1-dev`, `libwebkit2gtk-4.1-dev`, `libasound2-dev`, and others —
  see README.md for the full apt command). The Rust code has been written
  and manually reviewed for correctness but is **unverified by the
  compiler**. Whoever picks this up next should run `cargo check` in
  `src-tauri/` first and fix whatever compile errors surface — treat the
  Rust code as a first draft, not a known-working baseline.
- This was all developed inside WSL2 Linux. Tauri's webview and real audio
  devices don't exist there — `npm run tauri dev` and any actual
  microphone/speaker/OpenAI round-trip testing has never been run. That can
  only happen on a native Windows machine (the app's actual target
  platform). See README.md's "What still needs to be verified on Windows"
  section for the full list.

## 3. Tech stack & key dependency versions

- **Frontend**: React 19, TypeScript, Vite 7, Zustand 5 (`package.json`).
- **Backend**: Rust, Tauri 2. Key crates in `src-tauri/Cargo.toml`: `cpal`
  0.15 (audio I/O), `tokio` (async runtime, full features), `tokio-tungstenite`
  0.24 with `rustls-tls-webpki-roots` (WebSocket to OpenAI, no OpenSSL
  system dependency), `base64` 0.22, `tokio-util` 0.7 (`CancellationToken`),
  `thiserror` 2, `rubato` 0.16 (declared but not currently used — see §7).

## 4. Core architecture decision

**All audio I/O and the OpenAI Realtime WebSocket connection live entirely
in Rust (`src-tauri`). The React frontend never touches raw audio or the
OpenAI API key** — it only calls Tauri commands and listens for Tauri
events. This was a deliberate choice, not an oversight:

- CPAL capture/playback and a future virtual-mic sink can only be done from
  native code, so building it in Rust now avoids re-architecting later.
- The OpenAI API key never crosses the Tauri IPC boundary into webview JS,
  so it can't be scraped from the frontend.
- Only low-frequency data (connection status, transcript text) crosses IPC
  as Tauri events; raw PCM audio never does, keeping IPC overhead low.

The spec's `src/audio/*.ts` and `src/realtime/*.ts` files exist but are
**thin `invoke()`/`listen()` wrappers and shared TypeScript types only** —
there is no real audio or WebSocket code in the frontend at all.

## 5. The Tauri IPC contract (the seam between frontend and backend)

This is the actual interface — read this section before changing either side.

### Commands (frontend calls via `invoke()`, defined with `#[tauri::command]`)

| Command | Defined in | Signature | Notes |
|---|---|---|---|
| `list_input_devices` | `audio.rs` | `() -> Result<Vec<AudioDeviceInfo>, String>` | |
| `list_output_devices` | `audio.rs` | `() -> Result<Vec<AudioDeviceInfo>, String>` | |
| `get_settings` | `settings.rs` | `() -> Result<AppSettingsView, String>` | Never includes the raw API key. |
| `save_settings` | `settings.rs` | `(settings: AppSettingsInput) -> Result<(), String>` | Partial update; `None` fields are left unchanged on disk. |
| `start_session` | `session.rs` | `(input_device_id, output_device_id, source_lang, target_lang: String) -> Result<(), String>` | Errors if a session is already running. |
| `stop_session` | `session.rs` | `() -> Result<(), String>` | Cancels and awaits full cleanup. |
| `get_session_status` | `session.rs` | `() -> Result<bool, String>` | Whether a session is currently running. Written but not yet called from the frontend. |

All registered in `lib.rs`'s `tauri::generate_handler![...]`. Tauri v2
auto-converts JS camelCase call args to Rust snake_case params, so JS call
sites use camelCase keys (e.g. `invoke("start_session", { inputDeviceId, outputDeviceId, sourceLang, targetLang })`).

### Events (Rust emits via `app.emit()`, frontend subscribes via `listen()`)

Constants for event names live in `src/realtime/websocket.ts`'s `TAURI_EVENTS`.
Emitter helpers live in `session.rs` (`emit_status`, `emit_error`, `emit_transcript`).

| Event | Payload shape | Emitted from |
|---|---|---|
| `connection-status` | `{ status: "idle"\|"connecting"\|"connected"\|"reconnecting"\|"disconnected"\|"error", message?: string }` | `session.rs` |
| `transcript-update` | `{ id: string, role: "source"\|"translated", text: string, isFinal: boolean, timestamp: number }` | `session.rs` |
| `session-error` | `{ message: string, recoverable: boolean }` | `session.rs` |

Frontend subscription happens in one place only: `src/hooks/useTauriEvents.ts`,
called once from `App.tsx`.

## 6. Frontend file-by-file

- **`src/main.tsx`** — Vite/React entry point, mounts `<App />`.
- **`src/App.tsx`** — top-level component. Calls `useTauriEvents()` (wires
  up event listeners), calls `loadSettings()` on mount, and sets the
  `data-theme` attribute on `<html>` from the store's `theme` value (drives
  `App.css`'s CSS-variable theming). Renders `<MainPage />`.
- **`src/App.css`** — all styling. Light/dark theme via CSS custom
  properties (`:root` defaults, `:root[data-theme="dark"]` override, plus a
  `prefers-color-scheme` media query fallback). Plain CSS, no framework.
- **`src/pages/MainPage.tsx`** — composes the whole single-screen UI per the
  spec's mockup: two `DeviceSelector`s, `LanguagePairSelector`,
  `StatusIndicator`, `TranscriptPanel`, `SessionControls`, and a settings
  gear icon that opens `SettingsModal`.
- **`src/components/DeviceSelector.tsx`** — generic `<select>` for a device
  list. Props: `{ label, devices, selectedId, onChange, disabled? }`.
- **`src/components/LanguagePairSelector.tsx`** — shows source/target
  language with a swap button. Props: `{ sourceLanguage, targetLanguage, onSwap, disabled? }`.
- **`src/components/StatusIndicator.tsx`** — colored dot + label for
  connection status. Props: `{ status, errorMessage? }`.
- **`src/components/TranscriptPanel.tsx`** — scrolling list of transcript
  lines, auto-scrolls to bottom on new entries. Props: `{ entries }`.
- **`src/components/SessionControls.tsx`** — Start/Stop buttons; disabled
  states derived from `status`. Props: `{ status, onStart, onStop }`.
- **`src/settings/SettingsModal.tsx`** — modal with API key input (write-only,
  masked, calls `setApiKey`) and a theme `<select>`. Props: `{ onClose }`.
- **`src/store/useAppStore.ts`** — the single Zustand store; see §8.
- **`src/hooks/useDevices.ts`** — wraps device state, calls `refreshDevices()`
  on mount.
- **`src/hooks/useSession.ts`** — wraps session/status/transcript state and
  `startSession`/`stopSession` actions.
- **`src/hooks/useSettings.ts`** — wraps settings state, calls `loadSettings()`
  on mount.
- **`src/hooks/useTauriEvents.ts`** — the *only* place `listen()` is called;
  subscribes to all three Tauri events and dispatches into the store, with
  `unlisten()` cleanup on unmount.
- **`src/services/tauriClient.ts`** — two generic helpers used by everything
  else: `invokeCommand<T>(cmd, args?)` and `subscribeEvent<T>(event, callback)`.
- **`src/audio/microphone.ts`** — `listInputDevices()`, a one-line `invoke()`
  wrapper. No capture logic (that's Rust-only).
- **`src/audio/speaker.ts`** — `listOutputDevices()`, same pattern.
- **`src/audio/virtualMic.ts`** — **Phase 2 stub.** `isVirtualMicSupported()`
  currently always returns `false`. Not wired into any UI yet.
- **`src/realtime/openai.ts`** — `startRealtimeSession(params)` /
  `stopRealtimeSession()`, `invoke()` wrappers around `start_session`/`stop_session`.
- **`src/realtime/websocket.ts`** — **no WebSocket client lives here.**
  This file is repurposed as the shared protocol-types module: `ConnectionStatus`,
  `ConnectionStatusEvent`, `TranscriptEvent`, `SessionErrorEvent`,
  `AudioDeviceInfo`, `SessionStartParams`, and the `TAURI_EVENTS` name
  constants. Kept under this filename because it's the natural home for
  "WebSocket-related types," even though the actual socket is Rust-side.
- **`src/translation/language.ts`** — `LanguageCode` (`"en" | "ml"`),
  `SUPPORTED_LANGUAGES` array, `languageLabel(code)` helper. UI-only.
- **`src/translation/translator.ts`** — deliberately near-empty. The real
  translation prompt/config lives once, in Rust (`websocket.rs`), to avoid
  duplicating drift-prone logic in two languages.

## 7. Backend file-by-file (`src-tauri/src/`)

- **`main.rs`** — binary entry point, calls `voxbridge_lib::run()`.
- **`lib.rs`** — the wiring file. Declares all modules (`mod audio; mod microphone; mod session; mod settings; mod virtual_mic; mod websocket;`),
  builds the `tauri::Builder`, calls `.manage(SessionManager::default())`,
  and registers every `#[tauri::command]` in `generate_handler![...]`.
  **When adding a new command anywhere, it must also be added to the
  `generate_handler!` list here or it won't be callable.**
- **`audio.rs`** — device enumeration + DSP helpers + output playback.
  - `TARGET_SAMPLE_RATE: u32 = 24000` — the mono PCM16 rate OpenAI's
    Realtime API is assumed to use. Referenced by both capture and playback
    resampling.
  - `AudioDeviceInfo { id, name, is_default }` — serialized to the
    frontend. `id` is just the CPAL device name string (CPAL has no stable
    cross-platform device-id concept; acceptable for a single-user MVP).
  - `input_devices()` / `output_devices()` — pure enumeration functions.
  - `find_input_device(id)` / `find_output_device(id)` — look up a `cpal::Device`
    by the name-as-id.
  - `list_input_devices` / `list_output_devices` — the actual `#[tauri::command]`
    wrappers around the above.
  - `downmix_to_mono(data, channels)` — averages interleaved multi-channel
    samples to mono.
  - `resample_linear_f32` / `resample_linear_i16` — deliberately simple
    linear-interpolation resamplers (not windowed-sinc). Chosen over the
    declared-but-unused `rubato` dependency because correctly wiring
    rubato's chunked streaming API carried real risk of subtle bugs that
    can't be caught without a real audio device to test against. **`rubato`
    is in `Cargo.toml` but currently unused** — either wire it in for
    better quality once this can be tested on Windows, or remove the
    dependency.
  - `f32_to_pcm16_le_bytes` / `pcm16_le_bytes_to_i16` — format conversion
    helpers.
  - `PlaybackStream` — owns a CPAL output stream on a **dedicated OS thread**
    (not inside the async runtime — CPAL's `Stream` isn't reliably `Send`,
    so it can't be held across `.await` points). Audio to play is handed
    over via a shared `Arc<Mutex<VecDeque<i16>>>` ring buffer, not by
    moving the stream. `start(device_id)`, `push_samples(pcm16_mono)`
    (resamples from 24kHz to the device's native rate, replicates across
    channels), `stop()`.
- **`microphone.rs`** — `MicrophoneCapture`, the input-side mirror of
  `PlaybackStream`: same dedicated-thread pattern, `start(device_id, pcm_tx)`
  spawns a thread owning the CPAL input stream, whose callback downmixes,
  resamples to 24kHz, converts to PCM16-LE bytes, and pushes each chunk
  through a `tokio::sync::mpsc::UnboundedSender<Vec<u8>>`. `stop()` signals
  the thread to exit and joins it.
- **`websocket.rs`** — the OpenAI Realtime API client. **Contains the parts
  most likely to have drifted from OpenAI's actual current API** — see the
  file's top-of-file comment.
  - Constants: `REALTIME_URL_BASE = "wss://api.openai.com/v1/realtime"`,
    `REALTIME_MODEL = "gpt-4o-realtime-preview-2024-12-17"`,
    `OPENAI_BETA_HEADER_VALUE = "realtime=v1"`.
  - `client_events` / `server_events` modules — named string constants for
    every event `type` field used, so a doc-drift fix is one-place.
  - `RealtimeClient` — `connect(api_key)` (sets `Authorization: Bearer` and
    `OpenAI-Beta` headers, connects via `tokio-tungstenite`),
    `send_session_update(source_lang_name, target_lang_name)` (builds and
    sends the `session.update` payload — see §9 for the translation prompt),
    `send_audio_chunk(pcm16_le_bytes)` (base64-encodes and sends
    `input_audio_buffer.append`), `next_event()` (reads/parses the next
    server message as `serde_json::Value`; events are parsed generically
    rather than into strongly-typed structs, to reduce breakage risk from
    API drift), `close()`.
  - `decode_audio_delta(value)` / `extract_transcript_delta(value)` — pull
    the `delta` field out of a parsed server event.
- **`session.rs`** — the orchestrator tying capture, the WebSocket client,
  and playback together.
  - `SessionManager` — Tauri-managed state (`.manage()`'d in `lib.rs`),
    holds `current: tokio::sync::Mutex<Option<SessionHandle>>`.
  - `SessionHandle` — `{ cancel: CancellationToken, task: tauri::async_runtime::JoinHandle<()> }`.
  - `run_session(app, args, cancel)` — outer loop: reads the API key fresh
    each attempt (fails fast with a `session-error` if missing), calls
    `run_session_once`, and on error retries with exponential backoff
    (`(1u64 << attempt).min(30)` seconds) up to `MAX_RECONNECT_ATTEMPTS = 5`,
    emitting `reconnecting` status each attempt.
  - `run_session_once(app, args, api_key, cancel)` — connects, sends the
    session update, spawns mic capture and playback (via `spawn_blocking`
    since their `start()` calls block synchronously waiting for the CPAL
    thread to report readiness), then runs a `tokio::select!` loop reading
    three sources concurrently: cancellation, mic PCM chunks (forwarded to
    the WebSocket), and server events (dispatched to `handle_server_event`).
    Cleans up mic/playback/socket on any exit path.
  - `handle_server_event(app, playback, value)` — routes a parsed server
    event by its `type` field: audio deltas → `playback.push_samples`,
    transcript deltas → `transcript-update` (role `"translated"`),
    input-transcription-completed → `transcript-update` (role `"source"`),
    errors → `session-error`.
  - `language_name(code)` — maps `"en"`/`"ml"` to `"English"`/`"Malayalam"`
    for the prompt text; falls through to the raw code otherwise.
  - Commands: `start_session`, `stop_session`, `get_session_status` (table
    in §5).
- **`settings.rs`** — local persistence, hand-rolled (not `tauri-plugin-store`,
  deliberately — see the file's doc comment: the plugin's main value is a
  *frontend-facing* JS API, which is exactly what must not exist for the
  API key).
  - `StoredSettings` — on-disk shape: `api_key: Option<String>`,
    `input_language`, `output_language`, `theme: String`,
    `preferred_input_device`/`preferred_output_device: Option<String>`.
    Custom `Default` impl (`ml`/`en`/`dark` defaults), `#[serde(default)]`
    so old/partial JSON files still deserialize.
  - `AppSettingsInput` — write-only partial-update shape from the frontend,
    all fields `Option<...>`, camelCase via `#[serde(rename_all = "camelCase")]`.
  - `AppSettingsView` — safe read shape sent to the frontend: `has_api_key: bool`
    instead of the raw key, plus `theme`/`input_language`/`output_language`.
  - `settings_path(app)` — resolves `<app_config_dir>/settings.json`,
    creating the directory if needed.
  - `load(app)` / `save(app, &settings)` — read/write the JSON file,
    falling back to `StoredSettings::default()` on any read/parse failure.
  - `get_api_key(app)` — internal-only helper (not a Tauri command) used by
    `session.rs` to read the raw key; never exposed to the frontend.
  - Commands: `get_settings`, `save_settings` (table in §5).
- **`virtual_mic.rs`** — **Phase 2 extension point, not implemented.**
  `VirtualMicOutput` trait (`write_frames(&mut self, pcm16_mono: &[i16]) -> Result<(), VirtualMicError>`),
  `NoopVirtualMic` default impl. `SessionManager` does **not** currently
  hold one of these (the plan called for `Option<Box<dyn VirtualMicOutput>>`
  but it hasn't been wired in yet since virtual mic output isn't built).
  When Phase 2 adds a real VB-CABLE-backed implementation, it should slot in
  as another sink alongside `PlaybackStream` in `session.rs`'s event
  handling, without changing the session's control flow.

## 8. Zustand store shape (`src/store/useAppStore.ts`)

Single store, one file, no slices/middleware. Roughly:

```
inputDevices, outputDevices: AudioDeviceInfo[]
selectedInputDeviceId, selectedOutputDeviceId: string | null
refreshDevices(), setInputDevice(id), setOutputDevice(id)

sourceLanguage, targetLanguage: LanguageCode   // default ml -> en
swapLanguages()

status: ConnectionStatus                        // mirrors connection-status events
errorMessage: string | null
setStatus(status, message?)
startSession()   // validates devices selected, calls realtime/openai.ts, sets status
stopSession()

transcriptEntries: TranscriptEvent[]
appendTranscript(entry), clearTranscript()

theme: "light" | "dark"
hasApiKey: boolean
setApiKey(key)    // calls save_settings, updates hasApiKey locally
setTheme(theme)   // calls save_settings, updates theme locally
loadSettings()    // calls get_settings, hydrates theme/hasApiKey/languages
```

No raw audio, PCM buffers, the API key value, or reconnect counters ever
enter this store — those stay entirely Rust-side.

## 9. The translation approach, and its known risk

OpenAI's Realtime API is a conversational speech-to-speech model — it has
no dedicated "translate" mode. `websocket.rs`'s `send_session_update` works
around this with an `instructions` system prompt telling the model to act
as a direct interpreter and never answer/converse (full text in that
function). **This is flagged as a real correctness risk, not just
plumbing**: the model can be tempted to answer a question spoken in the
source audio instead of translating it. This needs testing with real
speech on Windows and will likely need prompt iteration. If prompt-only
translation proves unreliable, the documented fallback (not yet built) is a
two-stage pipeline: transcribe via `input_audio_transcription` → translate
the text separately → TTS.

## 10. Things very likely to need fixing once `cargo check` finally runs

Since the Rust code has never compiled in this environment, expect some of
these on the first real `cargo check`:

- `tokio-tungstenite` 0.24's exact `Message::Text` payload type (`String` vs
  `Utf8Bytes` in newer versions) — `websocket.rs` was written to be
  tolerant of either via `.into()` / `.as_ref()`, but verify.
- `tauri::async_runtime::JoinHandle` — used in `session.rs`'s `SessionHandle`;
  confirm this type still exists with this name in the pinned Tauri version.
- Exact cpal 0.15 API surface (`SupportedStreamConfig::config()`,
  `build_input_stream`/`build_output_stream`'s `Option<Duration>` timeout
  param) used throughout `audio.rs`/`microphone.rs`.
- Whether `rubato` (declared, unused) should be removed or actually wired
  in — see §7.

## 11. Where to look for more context

- `VoxBridge_MVP_Specification.md` — original product spec (full MVP scope,
  future phases, UI mockup).
- `README.md` — Windows setup/run instructions, prerequisites, and the
  "what still needs to be verified" checklist (audio/UX-focused; overlaps
  with §2/§10 here but from a testing rather than code perspective).
- `CLAUDE.md` — repo-specific instruction: don't add `Co-Authored-By: Claude`
  trailers to commits in this repo.
