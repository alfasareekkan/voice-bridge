import type { ConnectionStatus } from "../realtime/websocket";

interface SessionControlsProps {
  status: ConnectionStatus;
  onStart: () => void;
  onStop: () => void;
}

export function SessionControls({ status, onStart, onStop }: SessionControlsProps) {
  const isActive = status === "connecting" || status === "connected" || status === "reconnecting";

  return (
    <div className="session-controls">
      <button type="button" className="primary-button" onClick={onStart} disabled={isActive}>
        Start
      </button>
      <button type="button" className="secondary-button" onClick={onStop} disabled={!isActive}>
        Stop
      </button>
    </div>
  );
}
