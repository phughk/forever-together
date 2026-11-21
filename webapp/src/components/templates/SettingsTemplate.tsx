import type { ReactNode } from "react";

/**
 * Layout of the settings screen: an optional header, optional notices and one narrow column of sections.
 * Viewport-fit constraints (1280x720 window, 390x844 phone): the default view must be understood without scrolling.
 * - notices: at most one banner line plus one error line.
 * - sections: at most three cards in the default view. The first is the main action ("Phone link", about 8rem off and
 *   14rem on); the second is the "Addon" card (a badge, a values line and one button, about 8rem, the folder field
 *   hidden unless needed); the last is a collapsed "Advanced" card of one row. Anything longer goes inside the collapsed card.
 * The column is capped at 40rem. A page that has no navigation (the not-linked screen) omits the header.
 */
export interface SettingsTemplateProps {
  header?: ReactNode;
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
