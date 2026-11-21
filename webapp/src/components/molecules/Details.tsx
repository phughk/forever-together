import type { ReactNode } from "react";

export interface DetailsProps {
  /** The always-visible line that opens and closes the section. */
  summary: ReactNode;
  children: ReactNode;
  defaultOpen?: boolean;
}

/** A collapsed "Details" section: explanation and rarely used controls live here, never in the default view. */
export function Details({ summary, children, defaultOpen = false }: DetailsProps) {
  return (
    <details className="ft-details" open={defaultOpen || undefined}>
      <summary>{summary}</summary>
      <div className="ft-details__body">{children}</div>
    </details>
  );
}
