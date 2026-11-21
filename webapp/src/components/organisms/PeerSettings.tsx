import { InfoTip } from "../atoms/InfoTip";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { ContactRow } from "../molecules/ContactRow";
import { InlineForm } from "../molecules/InlineForm";
import { Locked } from "../molecules/Locked";
import { NumberField } from "../molecules/NumberField";
import { SettingRow } from "../molecules/SettingRow";
import "../../styles/settings.css";

export interface PeerSettingsProps {
  /** Players to remember (server field `known_peers_size`). */
  knownPeersSize: number;
  /** Backup contacts (server field `trusted_peers`), full 64-hex keys. */
  trustedPeers: string[];
  onChangeKnownPeersSize: (size: number) => void | Promise<void>;
  onAddTrusted: (key: string) => void | Promise<void>;
  onRemoveTrusted: (key: string) => void | Promise<void>;
  /** Viewed on a phone: players to remember can be changed, backup contacts can only be read. */
  remote?: boolean;
  error?: string | null;
}

const KEY_PATTERN = /^[0-9a-fA-F]{64}$/;

/** "Players to remember" and "Backup contacts". Sits inside Advanced. */
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
    <>
      <SettingRow
        title="Players to remember"
        hint="How many players this app keeps between runs. Players for the dungeons you search are kept first."
      >
        <NumberField label="Players to remember" value={knownPeersSize} onSave={onChangeKnownPeersSize} />
      </SettingRow>

      <div className="ft-advanced-section">
        <div className="ft-advanced-section__head">
          <h3 className="ft-section-label">Backup contacts</h3>
          <InfoTip label="About Backup contacts">
            Other always-on copies of this app. They are asked for players only when nobody you remember answers. Built-in ones are not listed.
          </InfoTip>
        </div>
        {trustedPeers.length === 0 ? (
          <Text size="small" muted>
            None
          </Text>
        ) : (
          <div>
            {trustedPeers.map((k) => (
              <ContactRow key={k} contactKey={k} onRemove={remote ? undefined : (key) => void onRemoveTrusted(key)} />
            ))}
          </div>
        )}
        <Locked locked={remote} block>
          <InlineForm
            label="Add a backup contact"
            submitLabel="Add"
            mono
            clearOnSubmit
            disabled={remote}
            placeholder="Paste the full key (64 characters)"
            onSubmit={(v) => {
              if (KEY_PATTERN.test(v)) void onAddTrusted(v.toLowerCase());
            }}
          />
        </Locked>
      </div>
      {error && <Notice tone="bad">{error}</Notice>}
    </>
  );
}
