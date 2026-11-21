import type { Meta, StoryObj } from "@storybook/react-vite";
import { PeerSettings } from "./PeerSettings";

const key = (n: number) => String(n).repeat(64).slice(0, 64);

const meta = {
  title: "Organisms/PeerSettings (Players and backup contacts)",
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

export const NoBackupContacts: Story = {};
export const SomeBackupContacts: Story = { args: { trustedPeers: [key(1), key(2)] } };
export const ManyBackupContacts: Story = { args: { trustedPeers: [1, 2, 3, 4, 5, 6, 7, 8].map(key), knownPeersSize: 1000 } };
/** Phone: players to remember can change; contacts are read-only and the add field is locked. */
export const ViewedOnPhone: Story = { args: { remote: true, trustedPeers: [key(1)] } };
export const WithError: Story = { args: { error: "The backup contacts could not be saved." } };
