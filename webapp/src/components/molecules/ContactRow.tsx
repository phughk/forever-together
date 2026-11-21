import { useState } from "react";
import { Button } from "../atoms/Button";
import "../../styles/settings.css";

export const SHORT_ID_CHARS = 8;

export interface ContactRowProps {
  /** The full 64-hex public key. Only the first characters are shown. */
  contactKey: string;
  onRemove?: (key: string) => void;
}

/** A backup contact by its short id, with a button that copies the full key. */
export function ContactRow({ contactKey, onRemove }: ContactRowProps) {
  const [copied, setCopied] = useState(false);
  const short = contactKey.slice(0, SHORT_ID_CHARS);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(contactKey);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      /* clipboard blocked: nothing to do */
    }
  };
  return (
    <div className="ft-contact">
      <span className="ft-contact__id">{short}</span>
      <span className="ft-contact__actions">
        <Button size="small" variant="ghost" aria-label={`Copy full key of ${short}`} onClick={() => void copy()}>
          {copied ? "Copied" : "Copy"}
        </Button>
        {onRemove && (
          <Button size="small" variant="ghost" aria-label={`Remove ${short}`} onClick={() => onRemove(contactKey)}>
            Remove
          </Button>
        )}
      </span>
    </div>
  );
}
