import { useEffect } from "react";
import { useAppStore } from "../store/useAppStore";

export function useDevices() {
  const inputDevices = useAppStore((s) => s.inputDevices);
  const outputDevices = useAppStore((s) => s.outputDevices);
  const selectedInputDeviceId = useAppStore((s) => s.selectedInputDeviceId);
  const selectedOutputDeviceId = useAppStore((s) => s.selectedOutputDeviceId);
  const setInputDevice = useAppStore((s) => s.setInputDevice);
  const setOutputDevice = useAppStore((s) => s.setOutputDevice);
  const refreshDevices = useAppStore((s) => s.refreshDevices);

  useEffect(() => {
    refreshDevices();
  }, [refreshDevices]);

  return {
    inputDevices,
    outputDevices,
    selectedInputDeviceId,
    selectedOutputDeviceId,
    setInputDevice,
    setOutputDevice,
    refreshDevices,
  };
}
