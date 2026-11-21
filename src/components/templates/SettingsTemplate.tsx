import type { ReactNode } from "react";

/**
 * Layout of the settings screen: a header, optional notices and one narrow column of setting sections.
 * Constraints: each section is a card of its own and may be 1 to 10 rows tall. The column is capped at 40rem so
 * long keys (64 characters) wrap inside it.
 */
export interface SettingsTemplateProps {
  header: ReactNode;
  notices?: ReactNode;
  sections: ReactNode;
}

export function SettingsTemplate({ header, notices, sections }: SettingsTemplateProps) {
  return (
    <div className="ft-page">
      {header}
      {notices && <div className="ft-page__notices">{notices}</div>}
      <main className="ft-page__narrow">{sections}</main>
    </div>
  );
}
