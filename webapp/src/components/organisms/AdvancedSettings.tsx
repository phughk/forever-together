import { Badge } from "../atoms/Badge";
import { Card } from "../atoms/Card";
import { Details } from "../molecules/Details";
import { HistorySettings, type HistorySettingsProps } from "./HistorySettings";
import { PeerSettings, type PeerSettingsProps } from "./PeerSettings";
import "../../styles/settings.css";

export interface AdvancedSettingsProps
  extends Pick<PeerSettingsProps, "knownPeersSize" | "trustedPeers" | "onChangeKnownPeersSize" | "onAddTrusted" | "onRemoveTrusted" | "remote"> {
  onClearHistory: HistorySettingsProps["onClear"];
  historyError?: HistorySettingsProps["error"];
  peerError?: PeerSettingsProps["error"];
  defaultOpen?: boolean;
}

/** One collapsed "Advanced" card: search history first, then players to remember and backup contacts. The summary chip shows the state. */
export function AdvancedSettings(p: AdvancedSettingsProps) {
  const contacts = p.trustedPeers.length;
  return (
    <Card>
      <Details
        defaultOpen={p.defaultOpen}
        summary={
          <span className="ft-advanced__summary">
            <span className="ft-advanced__title">Advanced</span>
            <Badge>
              {p.knownPeersSize} remembered, {contacts} {contacts === 1 ? "contact" : "contacts"}
            </Badge>
          </span>
        }
      >
        <HistorySettings onClear={p.onClearHistory} remote={p.remote} error={p.historyError} />
        <PeerSettings
          knownPeersSize={p.knownPeersSize}
          trustedPeers={p.trustedPeers}
          remote={p.remote}
          error={p.peerError}
          onChangeKnownPeersSize={p.onChangeKnownPeersSize}
          onAddTrusted={p.onAddTrusted}
          onRemoveTrusted={p.onRemoveTrusted}
        />
      </Details>
    </Card>
  );
}
