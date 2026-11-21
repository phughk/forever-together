import { ISSUES_URL } from "../../lib/links";
import { SettingsIcon } from "../atoms/SettingsIcon";
import "../../styles/settings.css";

export interface ReportProblemLinkProps {
  /** Defaults to the project's issue form. */
  href?: string;
}

/** A quiet one-line link that opens the issue form in a new tab. */
export function ReportProblemLink({ href = ISSUES_URL }: ReportProblemLinkProps) {
  return (
    <a className="ft-reportlink" href={href} target="_blank" rel="noreferrer">
      <SettingsIcon name="flag" size={16} />
      Report a problem
    </a>
  );
}
