import { useEffect, useRef } from "react";
import type { TranscriptEvent } from "../realtime/websocket";

interface TranscriptColumnProps {
  label: string;
  code: string;
  entries: TranscriptEvent[];
  variant: "source" | "translated";
}

function TranscriptColumn({ label, code, entries, variant }: TranscriptColumnProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [entries.length]);

  return (
    <div className={`transcript-column transcript-column-${variant}`}>
      <div className="transcript-column-header">
        <span className="lang-badge">{code.toUpperCase()}</span>
        <span>{label}</span>
      </div>
      <div className="transcript-column-body" ref={scrollRef}>
        {entries.length === 0 && <p className="transcript-empty">No speech yet.</p>}
        {entries.map((entry) => (
          <p
            key={entry.id + entry.role}
            className={`chat-bubble chat-bubble-${variant}${entry.isFinal ? "" : " chat-bubble-interim"}`}
          >
            {entry.text}
            <span className="chat-bubble-meta">
              {new Date(entry.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
            </span>
          </p>
        ))}
      </div>
    </div>
  );
}

interface TranscriptPanelProps {
  entries: TranscriptEvent[];
  sourceLabel: string;
  sourceCode: string;
  targetLabel: string;
  targetCode: string;
}

export function TranscriptPanel({ entries, sourceLabel, sourceCode, targetLabel, targetCode }: TranscriptPanelProps) {
  const sourceEntries = entries.filter((e) => e.role === "source");
  const translatedEntries = entries.filter((e) => e.role === "translated");

  return (
    <div className="transcript-columns">
      <TranscriptColumn label={sourceLabel} code={sourceCode} entries={sourceEntries} variant="source" />
      <TranscriptColumn label={targetLabel} code={targetCode} entries={translatedEntries} variant="translated" />
    </div>
  );
}
