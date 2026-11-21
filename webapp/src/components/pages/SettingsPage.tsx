import type { ConnectionStatus, Settings } from "../../api/types";
import { Notice } from "../atoms/Notice";
import { AppHeader, type View } from "../organisms/AppHeader";
import { LanSettings, type PairingState } from "../organisms/LanSettings";
import { PeerSettings } from "../organisms/PeerSettings";
import { TokenSettings } from "../organisms/TokenSettings";
import { SettingsTemplate } from "../templates/SettingsTemplate";

export interface SettingsPageProps {
  settings: Settings;
  status: ConnectionStatus;
  pairing: PairingState;
  /** Viewed from another device: loopback-only settings are read-only. */
  remote?: boolean;
  /** Last failed change, in plain language. */
  error?: string | null;
  onNavigate: (view: View) => void;
  onLanModeChange: (on: boolean) => void | Promise<void>;
  onLanAddressChange?: (address: string) => void;
  onResetToken: () => void | Promise<void>;
  onKnownPeersSizeChange: (size: number) => void | Promise<void>;
  onAddTrusted: (key: string) => void | Promise<void>;
  onRemoveTrusted: (key: string) => void | Promise<void>;
}

/** The settings screen. It shows user settings only: LAN mode, token reset, known-peers size, trusted peers. */
export function SettingsPage(p: SettingsPageProps) {
  return (
    <SettingsTemplate
      header={<AppHeader view="settings" onNavigate={p.onNavigate} status={p.status} />}
      notices={
        p.error ? <Notice tone="bad">{p.error}</Notice> : undefined
      }
      sections={
        <>
          <LanSettings lanMode={p.settings.lan_mode} onChange={p.onLanModeChange} pairing={p.pairing} remote={p.remote} onAddressChange={p.onLanAddressChange} />
          <TokenSettings onReset={p.onResetToken} remote={p.remote} />
          <PeerSettings
            knownPeersSize={p.settings.known_peers_size}
            trustedPeers={p.settings.trusted_peers}
            remote={p.remote}
            onChangeKnownPeersSize={p.onKnownPeersSizeChange}
            onAddTrusted={p.onAddTrusted}
            onRemoveTrusted={p.onRemoveTrusted}
          />
        </>
      }
    />
  );
}
