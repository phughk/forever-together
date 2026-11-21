import type { ConnectionStatus as Status } from "../../api/types";
import { Button } from "../atoms/Button";
import { Heading, Text } from "../atoms/Typography";
import { ConnectionStatus } from "../molecules/ConnectionStatus";

export type View = "finder" | "settings";

export interface AppHeaderProps {
  view: View;
  onNavigate: (view: View) => void;
  status: Status;
}

export function AppHeader({ view, onNavigate, status }: AppHeaderProps) {
  return (
    <header className="ft-header">
      <div className="ft-header__brand">
        <Heading level={1} as="h1">
          forever-together
        </Heading>
        <Text size="small" muted>
          Peer-to-peer dungeon finder
        </Text>
      </div>
      <nav className="ft-header__nav" aria-label="Main">
        <Button aria-current={view === "finder" ? "page" : undefined} onClick={() => onNavigate("finder")}>
          Finder
        </Button>
        <Button aria-current={view === "settings" ? "page" : undefined} onClick={() => onNavigate("settings")}>
          Settings
        </Button>
      </nav>
      <ConnectionStatus status={status} />
    </header>
  );
}
