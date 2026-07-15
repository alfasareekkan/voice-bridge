import type { ConnectionStatus } from "../realtime/websocket";

const STATUS_LABEL: Record<ConnectionStatus, string> = {
  idle: "Idle",
  connecting: "Connecting…",
  connected: "Connected",
  reconnecting: "Reconnecting…",
  disconnected: "Disconnected",
  error: "Error",
};

interface StatusIndicatorProps {
  status: ConnectionStatus;
  errorMessage?: string | null;
}

export function StatusIndicator({ status, errorMessage }: StatusIndicatorProps) {
  return (
    <div className="field">
      <span className="field-label">Status</span>
      <div className={`status-row status-${status}`}>
        <span className="status-dot" aria-hidden="true" />
        <span>{STATUS_LABEL[status]}</span>
      </div>
      {status === "error" && errorMessage && <p className="status-error">{errorMessage}</p>}
    </div>
  );
}
