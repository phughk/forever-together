import type { ReactNode } from "react";

/**
 * Layout of the finder screen: a slim top row (the character strip on the left, the header on the right), then
 * optional notices, then one main area. The template places slots and knows nothing about their content.
 *
 * Layouts:
 *  - "list" (default): the main area uses the full page width (up to about 88rem). Used for the dungeon list,
 *    which lays itself out as a responsive grid of cards inside its own scroll region.
 *  - "focus": a single card for the current screen (first run, searching, ended, party) at a readable width
 *    (up to about 46rem), centred. At 1100px and wider the strip moves from the top row to a side column beside
 *    the card, so the screen is a two-pane layout and not a lone column.
 *
 * Viewport-fit limits (a 1280x720 window and a 390x844 phone): the main action of the screen must be visible
 * without scrolling the page, so the slots together may use at most the viewport height.
 *  - header: one row of 40px: a gear, plus a connection chip only when the connection is not fine.
 *  - notices: at most one or two short banner lines. Hidden when empty.
 *  - strip: at most about 5rem, for example the one-line character summary. Names can run to 24 + 24 characters
 *    (first name and surname) and wrap, so never rely on one line. Hidden when empty.
 *  - main: one card, or a few stacked cards, never more than the remaining height. Anything longer (a list of
 *    1 to 30 dungeons) scrolls inside its own region and never moves the page.
 */
export interface FinderTemplateProps {
  header: ReactNode;
  notices?: ReactNode;
  strip?: ReactNode;
  main: ReactNode;
  layout?: "list" | "focus";
}

export function FinderTemplate({ header, notices, strip, main, layout = "list" }: FinderTemplateProps) {
  return (
    <div className={`ft-page ft-page--${layout}`}>
      <div className="ft-page__top">
        {strip && <div className="ft-page__strip">{strip}</div>}
        {header}
      </div>
      {notices && <div className="ft-page__notices">{notices}</div>}
      <main className="ft-page__main">{main}</main>
    </div>
  );
}
