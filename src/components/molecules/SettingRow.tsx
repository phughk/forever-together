import type { ReactNode } from "react";

export interface SettingRowProps {
  title: string;
  description?: string;
  /** The control: a switch, a button, a field. */
  children: ReactNode;
}

export function SettingRow({ title, description, children }: SettingRowProps) {
  return (
    <div className="ft-setting">
      <div className="ft-setting__text">
        <span className="ft-setting__title">{title}</span>
        {description && <span className="ft-text ft-text--sm ft-text--muted">{description}</span>}
      </div>
      <div>{children}</div>
    </div>
  );
}
