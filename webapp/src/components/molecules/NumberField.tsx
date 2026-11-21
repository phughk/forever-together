import { useState, type FormEvent } from "react";
import { Button } from "../atoms/Button";
import { TextInput } from "../atoms/TextInput";

export interface NumberFieldProps {
  /** Accessible name; the visible label sits beside it in a setting row. */
  label: string;
  value: number;
  min?: number;
  onSave: (n: number) => void | Promise<void>;
  disabled?: boolean;
}

/** A number input with a Save button, no visible label of its own (a SettingRow supplies it). */
export function NumberField({ label, value, min = 1, onSave, disabled }: NumberFieldProps) {
  const [text, setText] = useState(String(value));
  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const n = Number.parseInt(text, 10);
    if (Number.isFinite(n) && n >= min) await onSave(n);
  };
  return (
    <form className="ft-form__row" onSubmit={submit}>
      <TextInput
        type="number"
        inputMode="numeric"
        aria-label={label}
        value={text}
        disabled={disabled}
        style={{ width: "6rem", flex: "none" }}
        onChange={(e) => setText(e.target.value)}
      />
      <Button type="submit" disabled={disabled || text.trim() === "" || text === String(value)}>
        Save
      </Button>
    </form>
  );
}
