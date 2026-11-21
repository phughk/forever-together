import type { SelectHTMLAttributes } from "react";

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectProps extends Omit<SelectHTMLAttributes<HTMLSelectElement>, "children"> {
  options: SelectOption[];
  mono?: boolean;
}

/** A native drop-down styled like the text input. */
export function Select({ options, mono, className, ...rest }: SelectProps) {
  const cls = ["ft-input", mono && "ft-input--mono", className].filter(Boolean).join(" ");
  return (
    <select className={cls} {...rest}>
      {options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  );
}
