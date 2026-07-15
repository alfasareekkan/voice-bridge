import { invokeCommand } from "../services/tauriClient";
import type { AudioDeviceInfo } from "../realtime/websocket";

export function listInputDevices(): Promise<AudioDeviceInfo[]> {
  return invokeCommand<AudioDeviceInfo[]>("list_input_devices");
}
