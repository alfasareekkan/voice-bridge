import { useState } from "react";
import { useSettings } from "../hooks/useSettings";

interface SettingsModalProps {
  onClose: () => void;
}

export function SettingsModal({ onClose }: SettingsModalProps) {
  const { theme, hasApiKey, setApiKey, setTheme } = useSettings();
  const [keyInput, setKeyInput] = useState("");
  const [saving, setSaving] = useState(false);

  async function handleSaveKey() {
    if (!keyInput) return;
    setSaving(true);
    try {
      await setApiKey(keyInput);
      setKeyInput("");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>Settings</h2>

        <label className="field">
          <span className="field-label">OpenAI API Key</span>
          <input
            className="field-control"
            type="password"
            placeholder={hasApiKey ? "•••••••••••••• (saved)" : "sk-..."}
            value={keyInput}
            onChange={(e) => setKeyInput(e.target.value)}
          />
        </label>
        <button type="button" className="secondary-button" onClick={handleSaveKey} disabled={saving || !keyInput}>
          Save Key
        </button>

        <label className="field">
          <span className="field-label">Theme</span>
          <select
            className="field-control"
            value={theme}
            onChange={(e) => setTheme(e.target.value as "light" | "dark")}
          >
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </label>

        <button type="button" className="primary-button" onClick={onClose}>
          Close
        </button>
      </div>
    </div>
  );
}
