import type { AudioDeviceInfo } from "../realtime/websocket";

interface DeviceSelectorProps {
  label: string;
  devices: AudioDeviceInfo[];
  selectedId: string | null;
  onChange: (id: string) => void;
  disabled?: boolean;
}

export function DeviceSelector({ label, devices, selectedId, onChange, disabled }: DeviceSelectorProps) {
  return (
    <label className="field">
      <span className="field-label">{label}</span>
      <select
        className="field-control"
        value={selectedId ?? ""}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled || devices.length === 0}
      >
        {devices.length === 0 && <option value="">No devices found</option>}
        {devices.map((d) => (
          <option key={d.id} value={d.id}>
            {d.name}
            {d.isDefault ? " (default)" : ""}
          </option>
        ))}
      </select>
    </label>
  );
}
