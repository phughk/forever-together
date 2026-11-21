import type { ConnectionStatus, Settings } from "../../api/types";
import { Notice } from "../atoms/Notice";
import { RemoteBanner } from "../molecules/RemoteBanner";
import { AdvancedSettings } from "../organisms/AdvancedSettings";
import { AppHeader, type View } from "../organisms/AppHeader";
import { LanSettings, type PairingState } from "../organisms/LanSettings";
import { SettingsTemplate } from "../templates/SettingsTemplate";

export interface SettingsPageProps {
  settings: Settings;
  status: ConnectionStatus;
  pairing: PairingState;
  /** Viewed from another device: desktop-only settings are locked. */
  remote?: boolean;
  /** Last failed change, in plain language. */
  error?: string | null;
  onNavigate: (view: View) => void;
  onLanModeChange: (on: boolean) => void | Promise<void>;
  onLanAddressChange?: (address: string) => void;
  onResetToken: () => void | Promise<void>;
  onClearHistory: () => void | Promise<void>;
  onKnownPeersSizeChange: (size: number) => void | Promise<void>;
  onAddTrusted: (key: string) => void | Promise<void>;
  onRemoveTrusted: (key: string) => void | Promise<void>;
  /** Open the Advanced section at first render (stories). */
  advancedOpen?: boolean;
}

/** The settings screen: the phone link, then one collapsed Advanced section. */
export function SettingsPage(p: SettingsPageProps) {
  const notices =
    p.remote || p.error ? (
      <>
        {p.remote && <RemoteBanner />}
        {p.error && <Notice tone="bad">{p.error}</Notice>}
      </>
    ) : undefined;
  return (
    <SettingsTemplate
      header={<AppHeader view="settings" onNavigate={p.onNavigate} status={p.status} />}
      notices={notices}
      sections={
        <>
          <LanSettings
            lanMode={p.settings.lan_mode}
            onChange={p.onLanModeChange}
            pairing={p.pairing}
            remote={p.remote}
            onAddressChange={p.onLanAddressChange}
            onResetToken={p.onResetToken}
          />
          <AdvancedSettings
            defaultOpen={p.advancedOpen}
            remote={p.remote}
            knownPeersSize={p.settings.known_peers_size}
            trustedPeers={p.settings.trusted_peers}
            onClearHistory={p.onClearHistory}
            onChangeKnownPeersSize={p.onKnownPeersSizeChange}
            onAddTrusted={p.onAddTrusted}
            onRemoveTrusted={p.onRemoveTrusted}
          />
        </>
      }
    />
  );
}
