import type { ReactNode } from "react";

/**
 * Layout of the finder screen. The template places slots and knows nothing about their content.
 *
 * Content constraints for the slots:
 *  - header: one row. Wraps below 30rem. Product name at most about 24 characters.
 *  - notices: short one or two line banners. Hidden when empty.
 *  - top: the live panel (search progress or party). Full width, one card. Hidden when empty.
 *  - left: the narrow column (about 22rem). Names can run to 24 + 24 characters (first name and surname)
 *    and wrap anywhere, so never rely on one line.
 *  - right: the wide column. Lists may be 1 to 30 rows.
 * Below 52rem the columns stack in the order top, left, right.
 */
export interface FinderTemplateProps {
  header: ReactNode;
  notices?: ReactNode;
  top?: ReactNode;
  left: ReactNode;
  right: ReactNode;
}

export function FinderTemplate({ header, notices, top, left, right }: FinderTemplateProps) {
  return (
    <div className="ft-page">
      {header}
      {notices && <div className="ft-page__notices">{notices}</div>}
      {top && <div className="ft-page__top">{top}</div>}
      <main className="ft-page__columns">
        <div className="ft-page__column">{left}</div>
        <div className="ft-page__column">{right}</div>
      </main>
    </div>
  );
}
