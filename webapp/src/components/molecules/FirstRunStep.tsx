import type { ReactNode } from "react";
import "../../styles/finder.css";

export interface FirstRunStepProps {
  n: number;
  /** The step's one-line instruction. */
  title: ReactNode;
  /** Greyed and inert until an earlier step is done. */
  pending?: boolean;
  children?: ReactNode;
}

/** One numbered step of the first run: a number, a short title and optional content under it. Render inside an `ol.ft-firststeps`. */
export function FirstRunStep({ n, title, pending, children }: FirstRunStepProps) {
  return (
    <li className={["ft-firststep", pending && "ft-firststep--pending"].filter(Boolean).join(" ")} aria-disabled={pending || undefined}>
      <span className="ft-firststep__num" aria-hidden="true">
        {n}
      </span>
      <span className="ft-firststep__title">{title}</span>
      {children && <div className="ft-firststep__body">{children}</div>}
    </li>
  );
}
