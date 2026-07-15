import { useEffect } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { subscribeEvent } from "../services/tauriClient";
import { useAppStore } from "../store/useAppStore";
import {
  TAURI_EVENTS,
  type ConnectionStatusEvent,
  type SessionErrorEvent,
  type TranscriptEvent,
} from "../realtime/websocket";

export function useTauriEvents() {
  const setStatus = useAppStore((s) => s.setStatus);
  const appendTranscript = useAppStore((s) => s.appendTranscript);

  useEffect(() => {
    const unlisteners: Promise<UnlistenFn>[] = [
      subscribeEvent<ConnectionStatusEvent>(TAURI_EVENTS.connectionStatus, (payload) => {
        setStatus(payload.status, payload.message);
      }),
      subscribeEvent<TranscriptEvent>(TAURI_EVENTS.transcriptUpdate, (payload) => {
        appendTranscript(payload);
      }),
      subscribeEvent<SessionErrorEvent>(TAURI_EVENTS.sessionError, (payload) => {
        setStatus("error", payload.message);
      }),
    ];

    return () => {
      unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, [setStatus, appendTranscript]);
}
