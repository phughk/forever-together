import type { ConnectionStatus as Status } from "../../api/types";
import "../../styles/party.css";
import { Button } from "../atoms/Button";
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

function BackIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      <path d="M19 12H5M11 6l-6 6 6 6" />
    </svg>
  );
}

/**
 * One slim row with only what carries information: a Settings gear on the finder, a Back arrow on Settings,
 * and a connection chip that appears only when the connection is not fine.
 */
export function AppHeader({ view, onNavigate, status }: AppHeaderProps) {
  return (
    <header className={`ft-header ft-header--${view}`}>
      {view === "settings" && (
        <Button className="ft-button--icon" aria-label="Back" onClick={() => onNavigate("finder")}>
          <BackIcon />
        </Button>
      )}
      <ConnectionStatus status={status} />
      {view === "finder" && (
        <Button className="ft-button--icon ft-header__gear" aria-label="Settings" onClick={() => onNavigate("settings")}>
          <GearIcon />
        </Button>
      )}
    </header>
  );
}
