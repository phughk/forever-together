import type { ReactNode } from "react";

/** A label and its value, one row of a read-only summary. */
export function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="ft-field">
      <span className="ft-field__label">{label}</span>
      <span className="ft-field__value">{children}</span>
    </div>
  );
}
