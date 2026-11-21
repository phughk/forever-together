import type { ReactNode } from "react";
import { InfoTip } from "../atoms/InfoTip";

export interface SettingRowProps {
  title: string;
  description?: string;
  /** Short text for an info tip beside the title. */
  hint?: string;
  /** The control: a switch, a button, a field. */
  children: ReactNode;
}

export function SettingRow({ title, description, hint, children }: SettingRowProps) {
  return (
    <div className="ft-setting">
      <div className="ft-setting__text">
        {hint ? (
          <span className="ft-setting__titlerow">
            <span className="ft-setting__title">{title}</span>
            <InfoTip label={`About ${title}`}>{hint}</InfoTip>
          </span>
        ) : (
          <span className="ft-setting__title">{title}</span>
        )}
        {description && <span className="ft-text ft-text--sm ft-text--muted">{description}</span>}
      </div>
      <div>{children}</div>
    </div>
  );
}
