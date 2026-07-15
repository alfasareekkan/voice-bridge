import { create } from "zustand";
import { listInputDevices } from "../audio/microphone";
import { listOutputDevices } from "../audio/speaker";
import { startRealtimeSession, stopRealtimeSession } from "../realtime/openai";
import { invokeCommand } from "../services/tauriClient";
import type { AudioDeviceInfo, ConnectionStatus, TranscriptEvent } from "../realtime/websocket";
import type { LanguageCode } from "../translation/language";

interface AppSettings {
  hasApiKey: boolean;
  theme: "light" | "dark";
  inputLanguage: LanguageCode;
  outputLanguage: LanguageCode;
}

interface AppState {
  // devices
  inputDevices: AudioDeviceInfo[];
  outputDevices: AudioDeviceInfo[];
  selectedInputDeviceId: string | null;
  selectedOutputDeviceId: string | null;
  refreshDevices: () => Promise<void>;
  setInputDevice: (id: string) => void;
  setOutputDevice: (id: string) => void;

  // language
  sourceLanguage: LanguageCode;
  targetLanguage: LanguageCode;
  swapLanguages: () => void;

  // session/connection
  status: ConnectionStatus;
  errorMessage: string | null;
  setStatus: (status: ConnectionStatus, message?: string) => void;
  startSession: () => Promise<void>;
  stopSession: () => Promise<void>;

  // transcript
  transcriptEntries: TranscriptEvent[];
  appendTranscript: (entry: TranscriptEvent) => void;
  clearTranscript: () => void;

  // settings
  theme: "light" | "dark";
  hasApiKey: boolean;
  setApiKey: (key: string) => Promise<void>;
  setTheme: (theme: "light" | "dark") => Promise<void>;
  loadSettings: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  inputDevices: [],
  outputDevices: [],
  selectedInputDeviceId: null,
  selectedOutputDeviceId: null,
  refreshDevices: async () => {
    const [inputDevices, outputDevices] = await Promise.all([
      listInputDevices(),
      listOutputDevices(),
    ]);
    set((state) => ({
      inputDevices,
      outputDevices,
      selectedInputDeviceId:
        state.selectedInputDeviceId ??
        inputDevices.find((d) => d.isDefault)?.id ??
        inputDevices[0]?.id ??
        null,
      selectedOutputDeviceId:
        state.selectedOutputDeviceId ??
        outputDevices.find((d) => d.isDefault)?.id ??
        outputDevices[0]?.id ??
        null,
    }));
  },
  setInputDevice: (id) => set({ selectedInputDeviceId: id }),
  setOutputDevice: (id) => set({ selectedOutputDeviceId: id }),

  sourceLanguage: "ml",
  targetLanguage: "en",
  swapLanguages: () =>
    set((state) => ({
      sourceLanguage: state.targetLanguage,
      targetLanguage: state.sourceLanguage,
    })),

  status: "idle",
  errorMessage: null,
  setStatus: (status, message) => set({ status, errorMessage: message ?? null }),
  startSession: async () => {
    const { selectedInputDeviceId, selectedOutputDeviceId, sourceLanguage, targetLanguage } =
      get();
    if (!selectedInputDeviceId || !selectedOutputDeviceId) {
      set({ status: "error", errorMessage: "Select an input and output device first." });
      return;
    }
    set({ status: "connecting", errorMessage: null });
    try {
      await startRealtimeSession({
        inputDeviceId: selectedInputDeviceId,
        outputDeviceId: selectedOutputDeviceId,
        sourceLang: sourceLanguage,
        targetLang: targetLanguage,
      });
    } catch (err) {
      set({ status: "error", errorMessage: String(err) });
    }
  },
  stopSession: async () => {
    try {
      await stopRealtimeSession();
    } finally {
      set({ status: "idle" });
    }
  },

  transcriptEntries: [],
  appendTranscript: (entry) =>
    set((state) => ({ transcriptEntries: [...state.transcriptEntries, entry] })),
  clearTranscript: () => set({ transcriptEntries: [] }),

  theme: "dark",
  hasApiKey: false,
  setApiKey: async (key) => {
    await invokeCommand("save_settings", { settings: { apiKey: key } });
    set({ hasApiKey: key.length > 0 });
  },
  setTheme: async (theme) => {
    await invokeCommand("save_settings", { settings: { theme } });
    set({ theme });
  },
  loadSettings: async () => {
    const settings = await invokeCommand<AppSettings>("get_settings");
    set({
      theme: settings.theme,
      hasApiKey: settings.hasApiKey,
      sourceLanguage: settings.inputLanguage,
      targetLanguage: settings.outputLanguage,
    });
  },
}));
