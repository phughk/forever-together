import type { InputHTMLAttributes, TextareaHTMLAttributes } from "react";

export interface TextInputProps extends InputHTMLAttributes<HTMLInputElement> {
  mono?: boolean;
  invalid?: boolean;
}

export function TextInput({ mono, invalid, className, ...rest }: TextInputProps) {
  const cls = ["ft-input", mono && "ft-input--mono", className].filter(Boolean).join(" ");
  return <input className={cls} aria-invalid={invalid || undefined} {...rest} />;
}

export interface TextAreaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  mono?: boolean;
  invalid?: boolean;
}

export function TextArea({ mono, invalid, className, ...rest }: TextAreaProps) {
  const cls = ["ft-input", mono && "ft-input--mono", className].filter(Boolean).join(" ");
  return <textarea className={cls} aria-invalid={invalid || undefined} {...rest} />;
}
