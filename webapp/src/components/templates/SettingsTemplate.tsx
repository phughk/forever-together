import type { ReactNode } from "react";

/**
 * Layout of the settings screen: an optional header (a Back arrow), optional notices and a responsive grid of
 * section cards. Viewport-fit constraints (1280x720 window, 390x844 phone): the default view must be understood
 * without scrolling.
 * - notices: at most one banner line plus one error line.
 * - sections: at most three cards in the default view. The first is the main action ("Phone link", about 8rem off and
 *   14rem on); the second is the "Addon" card (a badge, a values line and one button, about 8rem, the folder field
 *   hidden unless needed); the last is a collapsed "Advanced" card of one row. Anything longer goes inside the collapsed card.
 * The grid has one column on a phone and as many columns of at least 24rem as fit on a desktop, up to a maximum of
 * 64rem. `narrow` (the not-linked screen) keeps a single centred column of about 32rem. A page that has no
 * navigation (the not-linked screen) omits the header.
 */
export interface SettingsTemplateProps {
  header?: ReactNode;
  notices?: ReactNode;
  sections: ReactNode;
  narrow?: boolean;
}

export function SettingsTemplate({ header, notices, sections, narrow }: SettingsTemplateProps) {
  return (
    <div className="ft-page ft-page--settings">
      {header}
      {notices && <div className="ft-page__notices ft-page__notices--settings">{notices}</div>}
      <main className={narrow ? "ft-page__narrow" : "ft-page__sections"}>{sections}</main>
    </div>
  );
}
