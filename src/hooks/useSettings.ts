import { useEffect } from "react";
import { useAppStore } from "../store/useAppStore";

export function useSettings() {
  const theme = useAppStore((s) => s.theme);
  const hasApiKey = useAppStore((s) => s.hasApiKey);
  const setApiKey = useAppStore((s) => s.setApiKey);
  const setTheme = useAppStore((s) => s.setTheme);
  const loadSettings = useAppStore((s) => s.loadSettings);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  return { theme, hasApiKey, setApiKey, setTheme };
}
