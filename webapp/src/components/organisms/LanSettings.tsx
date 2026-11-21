import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Select } from "../atoms/Select";
import { Switch } from "../atoms/Toggle";
import { Text } from "../atoms/Typography";
import { PairingQr } from "../molecules/PairingQr";
import { SettingRow } from "../molecules/SettingRow";

export type PairingState =
  | { kind: "off" }
  | { kind: "loading" }
  | { kind: "ready"; url: string; address: string; addresses: string[] }
  /** The server refused: this window is not the desktop, only loopback may pair or change LAN mode. */
  | { kind: "forbidden" }
  | { kind: "error"; message: string };

export interface LanSettingsProps {
  lanMode: boolean;
  onChange: (on: boolean) => void | Promise<void>;
  pairing: PairingState;
  /** Choose which of this computer's addresses the pairing link uses. */
  onAddressChange?: (address: string) => void;
  /** The page runs on a phone or another device, so the switch is read-only. */
  remote?: boolean;
}

export function LanSettings({ lanMode, onChange, pairing, remote, onAddressChange }: LanSettingsProps) {
  return (
    <Card title="Phone access (LAN mode)">
      <SettingRow
        title="Allow phones on this network"
        description={remote ? "Change this on the desktop app." : "Off: only this computer can open the app."}
      >
        <Switch checked={lanMode} disabled={remote} onChange={(on) => void onChange(on)}>
          {lanMode ? "On" : "Off"}
        </Switch>
      </SettingRow>
      {lanMode && (
        <Notice tone="warn">
          The link between phone and computer is plain HTTP. On a network you do not trust, someone could read the pairing token.
          Turn LAN mode off when you are done.
        </Notice>
      )}
      {pairing.kind === "loading" && <Text muted>Preparing the pairing code...</Text>}
      {pairing.kind === "ready" && pairing.addresses.length > 1 && (
        <SettingRow
          title="Network address"
          description="This computer is on more than one network. Pick the one your phone is on (usually the same Wi-Fi)."
        >
          <Select
            mono
            aria-label="Network address"
            value={pairing.address}
            options={pairing.addresses.map((a) => ({ value: a, label: a }))}
            onChange={(e) => onAddressChange?.(e.target.value)}
          />
        </SettingRow>
      )}
      {pairing.kind === "ready" && <PairingQr url={pairing.url} />}
      {pairing.kind === "forbidden" && (
        <Notice tone="info">The pairing code is shown only in the desktop app, so a phone cannot pair another phone.</Notice>
      )}
      {pairing.kind === "error" && <Notice tone="bad">{pairing.message}</Notice>}
    </Card>
  );
}
