import { invokeCommand } from "../services/tauriClient";
import type { AudioDeviceInfo } from "../realtime/websocket";

export function listOutputDevices(): Promise<AudioDeviceInfo[]> {
  return invokeCommand<AudioDeviceInfo[]>("list_output_devices");
}
