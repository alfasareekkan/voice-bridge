import { DeviceSelector } from "../components/DeviceSelector";
import { LanguagePairSelector } from "../components/LanguagePairSelector";
import { StatusIndicator } from "../components/StatusIndicator";
import { TranscriptPanel } from "../components/TranscriptPanel";
import { SessionControls } from "../components/SessionControls";
import { useDevices } from "../hooks/useDevices";
import { useSession } from "../hooks/useSession";
import { useAppStore } from "../store/useAppStore";
import { languageLabel } from "../translation/language";
import { MicIcon, SpeakerIcon } from "../components/icons";

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

  const isActive = status === "connecting" || status === "connected" || status === "reconnecting";
  const inputDeviceName = inputDevices.find((d) => d.id === selectedInputDeviceId)?.name;

  return (
    <div className="live-translate-page">
      <section className="live-banner">
        <div className="live-banner-info">
          <StatusIndicator status={status} errorMessage={errorMessage} />
          <p className="live-banner-title">Live Translation Session</p>
          <p className="live-banner-sub">
            {isActive
              ? `Streaming from ${inputDeviceName ?? "selected input device"}`
              : "Start a session to begin streaming"}
          </p>
        </div>
        <div className="waveform" aria-hidden="true">
          {Array.from({ length: 28 }).map((_, i) => (
            <span key={i} style={{ animationDelay: `${(i % 7) * 0.09}s` }} />
          ))}
        </div>
      </section>

      <section className="live-toolbar">
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
        <LanguagePairSelector
          sourceLanguage={sourceLanguage}
          targetLanguage={targetLanguage}
          onSwap={swapLanguages}
          disabled={isActive}
        />
        <SessionControls status={status} onStart={startSession} onStop={stopSession} />
      </section>

      <TranscriptPanel
        entries={transcriptEntries}
        sourceLabel={languageLabel(sourceLanguage)}
        sourceCode={sourceLanguage}
        targetLabel={languageLabel(targetLanguage)}
        targetCode={targetLanguage}
      />
    </div>
  );
}
