import { useState } from "react";
import { DeviceSelector } from "../components/DeviceSelector";
import { LanguagePairSelector } from "../components/LanguagePairSelector";
import { StatusIndicator } from "../components/StatusIndicator";
import { TranscriptPanel } from "../components/TranscriptPanel";
import { SessionControls } from "../components/SessionControls";
import { SettingsModal } from "../settings/SettingsModal";
import { useDevices } from "../hooks/useDevices";
import { useSession } from "../hooks/useSession";
import { useAppStore } from "../store/useAppStore";
import { MicIcon, SettingsIcon, SpeakerIcon } from "../components/icons";

export function MainPage() {
  const {
    inputDevices,
    outputDevices,
    selectedInputDeviceId,
    selectedOutputDeviceId,
    setInputDevice,
    setOutputDevice,
  } = useDevices();
  const { status, errorMessage, startSession, stopSession, transcriptEntries } = useSession();
  const sourceLanguage = useAppStore((s) => s.sourceLanguage);
  const targetLanguage = useAppStore((s) => s.targetLanguage);
  const swapLanguages = useAppStore((s) => s.swapLanguages);
  const [settingsOpen, setSettingsOpen] = useState(false);

  const isActive = status === "connecting" || status === "connected" || status === "reconnecting";

  return (
    <main className="app-shell">
      <div className="app-card">
        <header className="app-header">
          <div className="app-brand">
            <div className="app-logo">
              <MicIcon size={18} />
            </div>
            <div className="app-titles">
              <h1>VoxBridge</h1>
              <p>Real-time voice translation</p>
            </div>
          </div>
          <button type="button" className="icon-button" onClick={() => setSettingsOpen(true)} aria-label="Settings">
            <SettingsIcon />
          </button>
        </header>

        <section>
          <p className="section-label">Audio devices</p>
          <div className="device-grid">
            <DeviceSelector
              label="Input mic"
              icon={<MicIcon size={14} />}
              devices={inputDevices}
              selectedId={selectedInputDeviceId}
              onChange={setInputDevice}
              disabled={isActive}
            />

            <DeviceSelector
              label="Output speaker"
              icon={<SpeakerIcon size={14} />}
              devices={outputDevices}
              selectedId={selectedOutputDeviceId}
              onChange={setOutputDevice}
              disabled={isActive}
            />
          </div>
        </section>

        <LanguagePairSelector
          sourceLanguage={sourceLanguage}
          targetLanguage={targetLanguage}
          onSwap={swapLanguages}
          disabled={isActive}
        />

        <StatusIndicator status={status} errorMessage={errorMessage} />

        <TranscriptPanel entries={transcriptEntries} />

        <SessionControls status={status} onStart={startSession} onStop={stopSession} />
      </div>

      {settingsOpen && <SettingsModal onClose={() => setSettingsOpen(false)} />}
    </main>
  );
}
