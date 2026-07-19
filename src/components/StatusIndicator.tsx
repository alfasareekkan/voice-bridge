import type { ConnectionStatus } from "../realtime/websocket";
import { AlertIcon } from "./icons";

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
      <div className={`status-pill status-${status}`}>
        <span className="status-dot" aria-hidden="true" />
        <span>{STATUS_LABEL[status]}</span>
      </div>
      {(status === "error" || status === "reconnecting") && errorMessage && (
        <p className="status-error-message">
          <AlertIcon />
          <span>{errorMessage}</span>
        </p>
      )}
    </div>
  );
}
