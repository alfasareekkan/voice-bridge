import { useEffect } from "react";
import { MainPage } from "./pages/MainPage";
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

  return <MainPage />;
}

export default App;
