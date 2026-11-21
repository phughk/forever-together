import type { ConnectionStatus as Status } from "../../api/types";
import "../../styles/party.css";
import { Button } from "../atoms/Button";
import { Heading } from "../atoms/Typography";
import { ConnectionStatus } from "../molecules/ConnectionStatus";

export type View = "finder" | "settings";

export interface AppHeaderProps {
  view: View;
  onNavigate: (view: View) => void;
  status: Status;
}

function GearIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      <circle cx="12" cy="12" r="3" />
      <path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3M5.3 5.3l2.1 2.1M16.6 16.6l2.1 2.1M18.7 5.3l-2.1 2.1M7.4 16.6l-2.1 2.1" />
    </svg>
  );
}

/** One row: the title and the online dot on the left, Finder and a Settings icon on the right. */
export function AppHeader({ view, onNavigate, status }: AppHeaderProps) {
  return (
    <header className="ft-header ft-header--one-row">
      <div className="ft-header__brand ft-header__brand--row">
        <Heading level={1} as="h1">
          forever-together
        </Heading>
        <ConnectionStatus status={status} />
      </div>
      <nav className="ft-header__nav" aria-label="Main">
        <Button aria-current={view === "finder" ? "page" : undefined} onClick={() => onNavigate("finder")}>
          Finder
        </Button>
        <Button
          className="ft-button--icon"
          aria-label="Settings"
          aria-current={view === "settings" ? "page" : undefined}
          onClick={() => onNavigate("settings")}
        >
          <GearIcon />
        </Button>
      </nav>
    </header>
  );
}
