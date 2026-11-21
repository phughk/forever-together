import { useId, useState, type FormEvent } from "react";
import { Button } from "../atoms/Button";
import { Notice } from "../atoms/Notice";
import { TextArea } from "../atoms/TextInput";
import { Label } from "../atoms/Typography";

export interface CodePasteFormProps {
  onSubmit: (code: string) => void | Promise<void>;
  busy?: boolean;
  /** Plain-language reason the last code was refused. */
  error?: string | null;
  initialValue?: string;
}

/** Label, text area and button for pasting the addon's character code. Inner spaces are kept (base45 uses them). */
export function CodePasteForm({ onSubmit, busy, error, initialValue = "" }: CodePasteFormProps) {
  const id = useId();
  const [value, setValue] = useState(initialValue);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    // Trim the ends only: a space inside the code is a valid character.
    void onSubmit(value.replace(/^\s+|\s+$/g, ""));
  };
  return (
    <form className="ft-form" onSubmit={submit}>
      <div>
        <Label htmlFor={id}>Character code from the addon</Label>
        <TextArea
          id={id}
          mono
          rows={3}
          value={value}
          invalid={!!error}
          placeholder="WFT1:..."
          spellCheck={false}
          autoCapitalize="off"
          autoComplete="off"
          onChange={(e) => setValue(e.target.value)}
        />
      </div>
      {error && <Notice tone="bad">{error}</Notice>}
      <Button type="submit" variant="primary" disabled={busy || value.trim() === ""}>
        {busy ? "Reading..." : "Use this code"}
      </Button>
    </form>
  );
}
