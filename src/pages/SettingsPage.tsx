import { useState } from "react";
import { useSettings } from "../hooks/useSettings";
import { MoonIcon, SunIcon } from "../components/icons";

export function SettingsPage() {
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
    <div className="settings-page">
      <h2>Settings</h2>

      <section className="settings-section">
        <p className="section-label">OpenAI API key</p>
        <label className="field">
          <span className="field-label">API key</span>
          <input
            className="field-control"
            type="password"
            placeholder={hasApiKey ? "•••••••••••••• (saved)" : "sk-..."}
            value={keyInput}
            onChange={(e) => setKeyInput(e.target.value)}
          />
        </label>
        <button type="button" className="secondary-button" onClick={handleSaveKey} disabled={saving || !keyInput}>
          Save key
        </button>
      </section>

      <section className="settings-section">
        <p className="section-label">Appearance</p>
        <div className="field">
          <span className="field-label">Theme</span>
          <div className="theme-toggle">
            <button
              type="button"
              className={theme === "light" ? "is-selected" : ""}
              onClick={() => setTheme("light")}
            >
              <SunIcon />
              Light
            </button>
            <button
              type="button"
              className={theme === "dark" ? "is-selected" : ""}
              onClick={() => setTheme("dark")}
            >
              <MoonIcon />
              Dark
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}
