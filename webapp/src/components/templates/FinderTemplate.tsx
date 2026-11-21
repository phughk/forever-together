import type { ReactNode } from "react";

/**
 * Layout of the finder screen: one main area and an optional compact strip. The template places slots and
 * knows nothing about their content. The page picks the content for the current screen.
 *
 * Viewport-fit limits (a 1280x720 window and a 390x844 phone): the main action of the screen must be visible
 * without scrolling the page, so the slots together may use at most the viewport height.
 *  - header: one row of about 3.5rem. Product name at most about 24 characters.
 *  - notices: at most one or two short banner lines. Hidden when empty.
 *  - strip: at most about 5rem, for example the one-line character summary. Names can run to 24 + 24 characters
 *    (first name and surname) and wrap, so never rely on one line. Hidden when empty.
 *  - main: one card, or a few stacked cards, never more than the remaining height. Anything longer (a list of
 *    1 to 30 dungeons) scrolls inside its own region and never moves the page.
 * The column is capped at 44rem and centred.
 */
export interface FinderTemplateProps {
  header: ReactNode;
  notices?: ReactNode;
  strip?: ReactNode;
  main: ReactNode;
}

export function FinderTemplate({ header, notices, strip, main }: FinderTemplateProps) {
  return (
    <div className="ft-page">
      {header}
      {notices && <div className="ft-page__notices">{notices}</div>}
      {strip && <div className="ft-page__strip">{strip}</div>}
      <main className="ft-page__main">{main}</main>
    </div>
  );
}
