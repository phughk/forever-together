import type { InputHTMLAttributes } from "react";

export type ToggleProps = Omit<InputHTMLAttributes<HTMLInputElement>, "type" | "onChange"> & {
  onChange?: (checked: boolean) => void;
};

/** A labelled on/off switch. */
export function Switch({ onChange, children, ...rest }: ToggleProps) {
  return (
    <label className="ft-switch">
      <input type="checkbox" role="switch" onChange={(e) => onChange?.(e.target.checked)} {...rest} />
      {children}
    </label>
  );
}

/** A bare checkbox. The caller supplies the label through `aria-label` or a wrapping label. */
export function Checkbox({ onChange, ...rest }: ToggleProps) {
  return (
    <span className="ft-check">
      <input type="checkbox" onChange={(e) => onChange?.(e.target.checked)} {...rest} />
    </span>
  );
}
