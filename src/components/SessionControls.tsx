import type { ConnectionStatus } from "../realtime/websocket";
import { PlayIcon, StopIcon } from "./icons";

interface SessionControlsProps {
  status: ConnectionStatus;
  onStart: () => void;
  onStop: () => void;
}

export function SessionControls({ status, onStart, onStop }: SessionControlsProps) {
  const isActive = status === "connecting" || status === "connected" || status === "reconnecting";

  return (
    <div className="session-controls">
      {isActive ? (
        <button type="button" className="record-button is-active" onClick={onStop}>
          <StopIcon size={16} />
          Stop session
        </button>
      ) : (
        <button type="button" className="record-button" onClick={onStart}>
          <PlayIcon size={18} />
          Start session
        </button>
      )}
    </div>
  );
}
