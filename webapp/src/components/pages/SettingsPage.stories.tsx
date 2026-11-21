import type { Meta, StoryObj } from "@storybook/react-vite";
import { userEvent, within } from "storybook/test";
import { makeSettings } from "../../api/fixtures";
import type { PairingState } from "../organisms/LanSettings";
import { SettingsPage } from "./SettingsPage";

const URL = "http://192.168.1.23:41873/#token=3f9a1c0be27d44a8b6d15e90c7a2f8e14b0d9c63a5e7f1280b4d6c9a1e3f5072";
const key = (n: number) => String(n).repeat(64).slice(0, 64);
const READY: PairingState = { kind: "ready", url: URL, address: "192.168.1.23", addresses: ["192.168.1.23"] };
const SEVERAL: PairingState = { kind: "ready", url: URL, address: "192.168.1.23", addresses: ["192.168.1.23", "10.0.0.7"] };

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

const addAPhone: Story["play"] = async ({ canvasElement }) => {
  await userEvent.click(within(canvasElement).getByRole("button", { name: "Add a phone" }));
};

/** Phone link off, Advanced collapsed: the whole screen fits a 1280x720 window. */
export const Defaults: Story = {};
export const PhoneLinkOnWithoutQr: Story = { args: { settings: makeSettings({ lan_mode: true }), pairing: READY } };
export const PhoneLinkOnWithQr: Story = { args: { settings: makeSettings({ lan_mode: true }), pairing: READY }, play: addAPhone };
export const SeveralAddresses: Story = { args: { settings: makeSettings({ lan_mode: true }), pairing: SEVERAL }, play: addAPhone };
export const AdvancedOpen: Story = { args: { advancedOpen: true } };
export const AdvancedOpenWithContacts: Story = {
  args: { advancedOpen: true, settings: makeSettings({ trusted_peers: [key(1), key(2), key(3)] }) },
};
export const ManyContactsLargeCache: Story = {
  args: { advancedOpen: true, settings: makeSettings({ known_peers_size: 1000, trusted_peers: [1, 2, 3, 4, 5, 6, 7, 8, 9].map(key) }) },
};
/** Opened on a phone: one banner, locks on desktop-only controls, Advanced read-only. */
export const ViewedOnPhone: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { remote: true, advancedOpen: true, settings: makeSettings({ lan_mode: true, trusted_peers: [key(1)] }), pairing: { kind: "forbidden" } },
};
export const ViewedOnPhoneCollapsed: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { remote: true, settings: makeSettings({ lan_mode: true }), pairing: { kind: "forbidden" } },
};
export const SaveRefused: Story = { args: { error: "That can only be changed in the desktop app." } };
export const PhoneWidthDesktopControls: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { settings: makeSettings({ lan_mode: true }), pairing: SEVERAL },
  play: addAPhone,
};
