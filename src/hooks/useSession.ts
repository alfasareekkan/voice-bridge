import { useAppStore } from "../store/useAppStore";

export function useSession() {
  const status = useAppStore((s) => s.status);
  const errorMessage = useAppStore((s) => s.errorMessage);
  const startSession = useAppStore((s) => s.startSession);
  const stopSession = useAppStore((s) => s.stopSession);
  const transcriptEntries = useAppStore((s) => s.transcriptEntries);

  return { status, errorMessage, startSession, stopSession, transcriptEntries };
}
