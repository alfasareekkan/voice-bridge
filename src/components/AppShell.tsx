import { useState, type ReactNode } from "react";
import { useAppStore } from "../store/useAppStore";
import { useSettings } from "../hooks/useSettings";
import { MainPage } from "../pages/MainPage";
import { DashboardPage } from "../pages/DashboardPage";
import { RecordingsPage } from "../pages/RecordingsPage";
import { MeetingsPage } from "../pages/MeetingsPage";
import { SettingsPage } from "../pages/SettingsPage";
import {
  DashboardIcon,
  MeetingsIcon,
  MicIcon,
  MoonIcon,
  RecordingsIcon,
  SearchIcon,
  SettingsIcon,
  SunIcon,
} from "./icons";
import type { ConnectionStatus } from "../realtime/websocket";

type ViewKey = "dashboard" | "recordings" | "live-translate" | "meetings" | "settings";

const NAV_ITEMS: { key: ViewKey; label: string; icon: ReactNode }[] = [
  { key: "dashboard", label: "Dashboard", icon: <DashboardIcon /> },
  { key: "recordings", label: "Recordings", icon: <RecordingsIcon /> },
  { key: "live-translate", label: "Live Translate", icon: <MicIcon /> },
  { key: "meetings", label: "Meetings", icon: <MeetingsIcon /> },
  { key: "settings", label: "Settings", icon: <SettingsIcon /> },
];

function engineStatusLabel(status: ConnectionStatus): string {
  if (status === "connected" || status === "connecting" || status === "reconnecting") {
    return "Voice engine active";
  }
  if (status === "error") {
    return "Voice engine error";
  }
  return "Voice engine idle";
}

export function AppShell() {
  const [activeView, setActiveView] = useState<ViewKey>("live-translate");
  const status = useAppStore((s) => s.status);
  const { theme, setTheme } = useSettings();

  return (
    <div className="shell">
      <aside className="sidebar">
        <div>
          <div className="sidebar-brand">
            <div className="app-logo">
              <MicIcon size={18} />
            </div>
            <div>
              <p className="sidebar-brand-name">VoxBridge</p>
              <p className={`sidebar-status status-${status}`}>
                <span className="status-dot" aria-hidden="true" />
                {engineStatusLabel(status)}
              </p>
            </div>
          </div>

          <nav className="sidebar-nav">
            {NAV_ITEMS.map((item) => (
              <button
                key={item.key}
                type="button"
                className={`nav-item${activeView === item.key ? " is-active" : ""}`}
                onClick={() => setActiveView(item.key)}
                aria-current={activeView === item.key ? "page" : undefined}
              >
                {item.icon}
                {item.label}
              </button>
            ))}
          </nav>
        </div>

        <div className="sidebar-footer">
          <button type="button" className="new-session-button" onClick={() => setActiveView("live-translate")}>
            + New Session
          </button>
          <div className="sidebar-links">
            <span className="sidebar-link">Account</span>
            <span className="sidebar-link">Help</span>
          </div>
        </div>
      </aside>

      <div className="shell-main">
        <header className="topbar">
          <div className="search-field">
            <SearchIcon size={15} />
            <input type="text" placeholder="Search transcripts…" aria-label="Search transcripts" />
          </div>
          <div className="topbar-actions">
            <button
              type="button"
              className="icon-button"
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
              aria-label="Toggle theme"
            >
              {theme === "dark" ? <MoonIcon /> : <SunIcon />}
            </button>
            <div className="avatar-circle" aria-hidden="true" />
          </div>
        </header>

        <div className="shell-content">
          {activeView === "dashboard" && <DashboardPage />}
          {activeView === "recordings" && <RecordingsPage />}
          {activeView === "live-translate" && <MainPage />}
          {activeView === "meetings" && <MeetingsPage />}
          {activeView === "settings" && <SettingsPage />}
        </div>
      </div>
    </div>
  );
}
