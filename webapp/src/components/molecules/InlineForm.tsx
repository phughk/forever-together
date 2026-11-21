import { useId, useState, type FormEvent } from "react";
import { Button } from "../atoms/Button";
import { TextInput } from "../atoms/TextInput";
import { Label } from "../atoms/Typography";

export interface InlineFormProps {
  label: string;
  submitLabel: string;
  onSubmit: (value: string) => void | Promise<void>;
  initialValue?: string;
  inputMode?: "text" | "numeric";
  type?: "text" | "number";
  mono?: boolean;
  placeholder?: string;
  disabled?: boolean;
  /** Clear the field after a successful submit. */
  clearOnSubmit?: boolean;
}

/** A label, one input and a submit button on one row. Used for "add trusted peer" and "known peers size". */
export function InlineForm({ label, submitLabel, onSubmit, initialValue = "", type = "text", clearOnSubmit, ...rest }: InlineFormProps) {
  const id = useId();
  const [value, setValue] = useState(initialValue);
  const submit = async (e: FormEvent) => {
    e.preventDefault();
    await onSubmit(value.trim());
    if (clearOnSubmit) setValue("");
  };
  return (
    <form onSubmit={submit}>
      <Label htmlFor={id}>{label}</Label>
      <div className="ft-form__row">
        <TextInput
          id={id}
          type={type}
          value={value}
          mono={rest.mono}
          inputMode={rest.inputMode}
          placeholder={rest.placeholder}
          disabled={rest.disabled}
          onChange={(e) => setValue(e.target.value)}
        />
        <Button type="submit" disabled={rest.disabled || value.trim() === ""}>
          {submitLabel}
        </Button>
      </div>
    </form>
  );
}
