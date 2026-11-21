import type { ReactNode } from "react";
import "../../styles/settings.css";

export interface NumberedStepProps {
  n: number;
  /** An icon that shows the step. */
  icon: ReactNode;
  children: ReactNode;
}

/** One numbered step with an icon and a short label. Render inside an `<ol className="ft-steps">`. */
export function NumberedStep({ n, icon, children }: NumberedStepProps) {
  return (
    <li className="ft-step">
      <span className="ft-step__num" aria-hidden="true">
        {n}
      </span>
      <span className="ft-step__icon">{icon}</span>
      <span className="ft-step__label">{children}</span>
    </li>
  );
}
