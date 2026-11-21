import type { Meta, StoryObj } from "@storybook/react-vite";
import { makeSettings } from "../../api/fixtures";
import { SettingsPage } from "./SettingsPage";

const URL = "http://192.168.1.23:41873/#token=3f9a1c0be27d44a8b6d15e90c7a2f8e14b0d9c63a5e7f1280b4d6c9a1e3f5072";
const key = (n: number) => String(n).repeat(64).slice(0, 64);

const meta = {
  title: "Pages/Settings",
  component: SettingsPage,
  parameters: { layout: "fullscreen" },
  args: {
    settings: makeSettings(),
    status: "live",
    pairing: { kind: "off" },
    onNavigate: () => {},
    onLanModeChange: () => {},
    onResetToken: () => {},
    onClearHistory: () => {},
    onKnownPeersSizeChange: () => {},
    onAddTrusted: () => {},
    onRemoveTrusted: () => {},
  },
} satisfies Meta<typeof SettingsPage>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Defaults: Story = {};
export const LanModeOnWithPairingQr: Story = {
  args: { settings: makeSettings({ lan_mode: true }), pairing: { kind: "ready", url: URL, address: "192.168.1.23", addresses: ["192.168.1.23", "10.0.0.7"] } },
};
export const WithTrustedPeers: Story = { args: { settings: makeSettings({ trusted_peers: [key(1), key(2), key(3)] }) } };
export const ManyTrustedPeersLargeCache: Story = {
  args: { settings: makeSettings({ known_peers_size: 1000, trusted_peers: [1, 2, 3, 4, 5, 6, 7, 8, 9].map(key) }) },
};
/** Opened on a phone: loopback-only settings are read-only and the pairing QR is not shown. */
export const ViewedOnPhone: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { remote: true, settings: makeSettings({ lan_mode: true, trusted_peers: [key(1)] }), pairing: { kind: "forbidden" } },
};
export const SaveRefused: Story = { args: { error: "That can only be changed in the desktop app." } };
export const PhoneWidthDesktopControls: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { settings: makeSettings({ lan_mode: true }), pairing: { kind: "ready", url: URL, address: "192.168.1.23", addresses: ["192.168.1.23", "10.0.0.7"] } },
};
