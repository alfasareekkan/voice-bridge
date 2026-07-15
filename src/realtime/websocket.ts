// Shared protocol types describing the Tauri command/event contract with the
// Rust backend. There is no WebSocket client in the frontend — the actual
// OpenAI Realtime WebSocket connection lives entirely in src-tauri/src/websocket.rs.
// This file only defines the shapes that cross the Tauri IPC boundary.

export type ConnectionStatus =
  | "idle"
  | "connecting"
  | "connected"
  | "reconnecting"
  | "disconnected"
  | "error";

export interface ConnectionStatusEvent {
  status: ConnectionStatus;
  message?: string;
}

export interface TranscriptEvent {
  id: string;
  role: "source" | "translated";
  text: string;
  isFinal: boolean;
  timestamp: number;
}

export interface SessionErrorEvent {
  message: string;
  recoverable: boolean;
}

export interface AudioDeviceInfo {
  id: string;
  name: string;
  isDefault: boolean;
}

export interface SessionStartParams {
  inputDeviceId: string;
  outputDeviceId: string;
  sourceLang: string;
  targetLang: string;
}

export const TAURI_EVENTS = {
  connectionStatus: "connection-status",
  transcriptUpdate: "transcript-update",
  sessionError: "session-error",
} as const;
