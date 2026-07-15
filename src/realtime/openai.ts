import { invokeCommand } from "../services/tauriClient";
import type { SessionStartParams } from "./websocket";

export function startRealtimeSession(params: SessionStartParams): Promise<void> {
  return invokeCommand<void>("start_session", {
    inputDeviceId: params.inputDeviceId,
    outputDeviceId: params.outputDeviceId,
    sourceLang: params.sourceLang,
    targetLang: params.targetLang,
  });
}

export function stopRealtimeSession(): Promise<void> {
  return invokeCommand<void>("stop_session");
}
