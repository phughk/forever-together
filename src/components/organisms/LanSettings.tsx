import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Switch } from "../atoms/Toggle";
import { Text } from "../atoms/Typography";
import { PairingQr } from "../molecules/PairingQr";
import { SettingRow } from "../molecules/SettingRow";

export type PairingState =
  | { kind: "off" }
  | { kind: "loading" }
  | { kind: "ready"; url: string }
  /** The server refused: this window is not the desktop, only loopback may pair or change LAN mode. */
  | { kind: "forbidden" }
  | { kind: "error"; message: string };

export interface LanSettingsProps {
  lanMode: boolean;
  onChange: (on: boolean) => void | Promise<void>;
  pairing: PairingState;
  /** The page runs on a phone or another device, so the switch is read-only. */
  remote?: boolean;
}

export function LanSettings({ lanMode, onChange, pairing, remote }: LanSettingsProps) {
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
      {pairing.kind === "ready" && <PairingQr url={pairing.url} />}
      {pairing.kind === "forbidden" && (
        <Notice tone="info">The pairing code is shown only in the desktop app, so a phone cannot pair another phone.</Notice>
      )}
      {pairing.kind === "error" && <Notice tone="bad">{pairing.message}</Notice>}
    </Card>
  );
}
