import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { InlineForm } from "../molecules/InlineForm";
import { PeerRow } from "../molecules/PeerRow";

export interface PeerSettingsProps {
  knownPeersSize: number;
  trustedPeers: string[];
  onChangeKnownPeersSize: (size: number) => void | Promise<void>;
  onAddTrusted: (key: string) => void | Promise<void>;
  onRemoveTrusted: (key: string) => void | Promise<void>;
  /** Viewed on a phone: trusted peers can be read but not changed. */
  remote?: boolean;
  error?: string | null;
}

const KEY_PATTERN = /^[0-9a-fA-F]{64}$/;

export function PeerSettings({
  knownPeersSize,
  trustedPeers,
  onChangeKnownPeersSize,
  onAddTrusted,
  onRemoveTrusted,
  remote,
  error,
}: PeerSettingsProps) {
  return (
    <Card title="Peers">
      <InlineForm
        label="Known peers to remember"
        submitLabel="Save"
        type="number"
        inputMode="numeric"
        initialValue={String(knownPeersSize)}
        onSubmit={(v) => {
          const n = Number.parseInt(v, 10);
          if (Number.isFinite(n) && n >= 1) void onChangeKnownPeersSize(n);
        }}
      />
      <Text size="small" muted>
        The app keeps this many peers between runs. Peers for the dungeons you are searching are kept first.
      </Text>

      <h3 className="ft-section-label">Extra trusted peers</h3>
      <Text size="small" muted>
        Trusted peers are ordinary peers. The app asks them for other peers only when no remembered peer answers. The built-in
        ones are not listed here.
      </Text>
      {trustedPeers.length === 0 ? (
        <Text size="small" muted>
          None added.
        </Text>
      ) : (
        <div>
          {trustedPeers.map((k) => (
            <PeerRow key={k} peerKey={k} onRemove={remote ? undefined : (key) => void onRemoveTrusted(key)} />
          ))}
        </div>
      )}
      {remote ? (
        <Notice tone="info">Trusted peers can be changed only in the desktop app.</Notice>
      ) : (
        <InlineForm
          label="Add a trusted peer (public key)"
          submitLabel="Add"
          mono
          clearOnSubmit
          placeholder="64 hex characters"
          onSubmit={(v) => {
            if (KEY_PATTERN.test(v)) void onAddTrusted(v.toLowerCase());
          }}
        />
      )}
      {error && <Notice tone="bad">{error}</Notice>}
    </Card>
  );
}
