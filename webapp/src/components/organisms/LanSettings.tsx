import { useEffect, useState } from "react";
import { Badge } from "../atoms/Badge";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Select } from "../atoms/Select";
import { Switch } from "../atoms/Toggle";
import { Text } from "../atoms/Typography";
import { Locked } from "../molecules/Locked";
import { PairingQr } from "../molecules/PairingQr";
import { SettingRow } from "../molecules/SettingRow";
import { TokenSettings } from "./TokenSettings";
import "../../styles/settings.css";

export type PairingState =
  | { kind: "off" }
  | { kind: "loading" }
  | { kind: "ready"; url: string; address: string; addresses: string[] }
  /** The server refused: this window is not the desktop, only loopback may pair or change the phone link. */
  | { kind: "forbidden" }
  | { kind: "error"; message: string };

export interface LanSettingsProps {
  /** Phone link on or off (the server field is still `lan_mode`). */
  lanMode: boolean;
  onChange: (on: boolean) => void | Promise<void>;
  pairing: PairingState;
  /** Choose which of this computer's addresses the link uses. Only offered when there is more than one. */
  onAddressChange?: (address: string) => void;
  /** The page runs on a phone or another device: every control is locked. */
  remote?: boolean;
  /** When given, a "Sign out all phones" row shows while the phone link is on. */
  onResetToken?: () => void | Promise<void>;
  /** Failure of the last sign-out, shown under its row. */
  resetError?: string | null;
  /** Start with the QR open (stories). */
  defaultShowQr?: boolean;
}

/** The "Phone link" card: the switch with its one-line risk, an "Add a phone" reveal for the QR, and sign out. */
export function LanSettings({ lanMode, onChange, pairing, remote, onAddressChange, onResetToken, resetError, defaultShowQr }: LanSettingsProps) {
  const [showQr, setShowQr] = useState(defaultShowQr ?? false);
  const open = lanMode && showQr && !remote;
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && setShowQr(false);
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [open]);
  return (
    <Card>
      <SettingRow
        title="Phone link"
        hint="Phones reach this PC over your home network without encryption. Anyone on it who gets your link could use the app. Switch off when you are done."
      >
        <div className="ft-phonelink__state">
          {lanMode && <Badge tone="warn">Not encrypted</Badge>}
          <Locked locked={remote}>
            <Switch checked={lanMode} disabled={remote} onChange={(on) => void onChange(on)}>
              {lanMode ? "On" : "Off"}
            </Switch>
          </Locked>
        </div>
      </SettingRow>
      {lanMode && <p className="ft-phonelink__risk">Anyone on this Wi-Fi could connect.</p>}
      {lanMode && !open && (
        <Locked locked={remote}>
          <Button variant="primary" disabled={remote} onClick={() => setShowQr(true)}>
            Add a phone
          </Button>
        </Locked>
      )}
      {open && (
        <div className="ft-sheet">
          <div className="ft-phonelink__reveal" role="dialog" aria-label="Add a phone">
            <div className="ft-phonelink__scroll">
              {pairing.kind === "loading" && <Text muted>Preparing the QR...</Text>}
              {pairing.kind === "ready" && (
                <>
                  <PairingQr url={pairing.url} />
                  <p className="ft-phonelink__value">
                    Phones connect to <code>{pairing.address}</code>
                  </p>
                  {pairing.addresses.length > 1 && (
                    <SettingRow title="Network" hint="This PC is on more than one network. Pick the one your phone is on, usually the same Wi-Fi.">
                      <Select
                        mono
                        aria-label="Network address"
                        value={pairing.address}
                        options={pairing.addresses.map((a) => ({ value: a, label: a }))}
                        onChange={(e) => onAddressChange?.(e.target.value)}
                      />
                    </SettingRow>
                  )}
                </>
              )}
            </div>
            <Button variant="primary" onClick={() => setShowQr(false)}>
              Done
            </Button>
          </div>
        </div>
      )}
      {lanMode && pairing.kind === "error" && <Notice tone="bad">{pairing.message}</Notice>}
      {lanMode && onResetToken && <TokenSettings onReset={onResetToken} remote={remote} error={resetError} />}
    </Card>
  );
}
