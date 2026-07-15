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
      <header className="app-header">
        <h1>VoxBridge</h1>
        <button type="button" className="icon-button" onClick={() => setSettingsOpen(true)} aria-label="Settings">
          ⚙
        </button>
      </header>

      <DeviceSelector
        label="Input Mic"
        devices={inputDevices}
        selectedId={selectedInputDeviceId}
        onChange={setInputDevice}
        disabled={isActive}
      />

      <DeviceSelector
        label="Output Speaker"
        devices={outputDevices}
        selectedId={selectedOutputDeviceId}
        onChange={setOutputDevice}
        disabled={isActive}
      />

      <LanguagePairSelector
        sourceLanguage={sourceLanguage}
        targetLanguage={targetLanguage}
        onSwap={swapLanguages}
        disabled={isActive}
      />

      <StatusIndicator status={status} errorMessage={errorMessage} />

      <TranscriptPanel entries={transcriptEntries} />

      <SessionControls status={status} onStart={startSession} onStop={stopSession} />

      {settingsOpen && <SettingsModal onClose={() => setSettingsOpen(false)} />}
    </main>
  );
}
