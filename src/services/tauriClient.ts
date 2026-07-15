import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args);
}

export function subscribeEvent<T>(
  event: string,
  callback: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, (e) => callback(e.payload));
}
