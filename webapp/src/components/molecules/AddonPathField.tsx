import { useId, useState, type FormEvent } from "react";
import { Button } from "../atoms/Button";
import { Notice } from "../atoms/Notice";
import { TextInput } from "../atoms/TextInput";
import { Label } from "../atoms/Typography";
import "../../styles/settings.css";

export interface AddonPathFieldProps {
  /** Called with the pasted path, or with a candidate when one is tapped. */
  onSubmit: (folder: string) => void | Promise<void>;
  /** Folders the app found. Each is a one-tap choice. */
  candidates?: string[];
  busy?: boolean;
  /** Plain-language reason the last folder was refused. */
  error?: string | null;
  initialValue?: string;
  /** Close the field without saving. */
  onCancel?: () => void;
}

const EXAMPLE = "/Applications/World of Warcraft/_forever_/Interface/AddOns";

/** A paste field for the game's AddOns folder with a "Use folder" button, and the folders the app found as one-tap choices. */
export function AddonPathField({ onSubmit, candidates = [], busy, error, initialValue = "", onCancel }: AddonPathFieldProps) {
  const id = useId();
  const [value, setValue] = useState(initialValue);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    void onSubmit(value.trim());
  };
  return (
    <form className="ft-addonpath" onSubmit={submit}>
      <div>
        <Label htmlFor={id}>AddOns folder</Label>
        <div className="ft-addonpath__row">
          <TextInput
            id={id}
            mono
            value={value}
            invalid={!!error}
            placeholder={EXAMPLE}
            spellCheck={false}
            autoCapitalize="off"
            autoComplete="off"
            onChange={(e) => setValue(e.target.value)}
          />
          <Button type="submit" variant="primary" disabled={busy || value.trim() === ""}>
            Use folder
          </Button>
          {onCancel && (
            <Button variant="ghost" onClick={onCancel} disabled={busy}>
              Cancel
            </Button>
          )}
        </div>
      </div>
      {error && <Notice tone="bad">{error}</Notice>}
      {candidates.length > 0 && (
        <div className="ft-addonpath__choices" role="group" aria-label="Folders found">
          <span className="ft-text ft-text--sm ft-text--muted">Found on this computer:</span>
          {candidates.map((c) => (
            <button key={c} type="button" className="ft-addonpath__choice" disabled={busy} onClick={() => void onSubmit(c)}>
              {c}
            </button>
          ))}
        </div>
      )}
    </form>
  );
}
