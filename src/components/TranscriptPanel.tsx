import { useEffect, useRef } from "react";
import type { TranscriptEvent } from "../realtime/websocket";

interface TranscriptPanelProps {
  entries: TranscriptEvent[];
}

export function TranscriptPanel({ entries }: TranscriptPanelProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [entries.length]);

  return (
    <div className="field">
      <span className="section-label">Transcript</span>
      <div className="transcript-panel" ref={scrollRef}>
        {entries.length === 0 && <p className="transcript-empty">No speech yet.</p>}
        {entries.map((entry) => (
          <p
            key={entry.id + entry.role}
            className={`transcript-line transcript-${entry.role}`}
          >
            {entry.text}
          </p>
        ))}
      </div>
    </div>
  );
}
