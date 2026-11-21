import type { Meta, StoryObj } from "@storybook/react-vite";
import { PeerSettings } from "./PeerSettings";

const key = (n: number) => String(n).repeat(64).slice(0, 64);

const meta = {
  title: "Organisms/PeerSettings",
  component: PeerSettings,
  args: {
    knownPeersSize: 100,
    trustedPeers: [],
    onChangeKnownPeersSize: () => {},
    onAddTrusted: () => {},
    onRemoveTrusted: () => {},
  },
} satisfies Meta<typeof PeerSettings>;
export default meta;
type Story = StoryObj<typeof meta>;

export const NoExtraTrustedPeers: Story = {};
export const SomeTrustedPeers: Story = { args: { trustedPeers: [key(1), key(2)] } };
export const ManyTrustedPeers: Story = { args: { trustedPeers: [1, 2, 3, 4, 5, 6, 7, 8].map(key), knownPeersSize: 1000 } };
export const ViewedOnPhone: Story = { args: { remote: true, trustedPeers: [key(1)] } };
export const WithError: Story = { args: { error: "The trusted peer list can only be changed on the desktop." } };
