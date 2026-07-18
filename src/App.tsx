import { useEffect } from "react";
import { AppShell } from "./components/AppShell";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useAppStore } from "./store/useAppStore";
import "./App.css";

function App() {
  const loadSettings = useAppStore((s) => s.loadSettings);
  const theme = useAppStore((s) => s.theme);
  useTauriEvents();

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  return <AppShell />;
}

export default App;
