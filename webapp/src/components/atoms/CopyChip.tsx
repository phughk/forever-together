import { useEffect, useState } from "react";
import "../../styles/finder.css";

export interface CopyChipProps {
  /** The text shown and copied. */
  text: string;
  className?: string;
}

/** Text in a chip that copies itself to the clipboard when pressed. */
export function CopyChip({ text, className }: CopyChipProps) {
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) return;
    const t = window.setTimeout(() => setCopied(false), 1500);
    return () => window.clearTimeout(t);
  }, [copied]);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
    } catch {
      // Clipboard blocked: the text stays visible and selectable, so nothing is lost.
    }
  };
  return (
    <button type="button" className={["ft-copychip", className].filter(Boolean).join(" ")} onClick={() => void copy()}>
      {text}
      <span className="ft-copychip__state" aria-live="polite">
        {copied ? "Copied" : "Copy"}
      </span>
    </button>
  );
}
