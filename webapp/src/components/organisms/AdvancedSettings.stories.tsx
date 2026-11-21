import type { Meta, StoryObj } from "@storybook/react-vite";
import { AdvancedSettings } from "./AdvancedSettings";

const key = (n: number) => String(n).repeat(64).slice(0, 64);

const meta = {
  title: "Organisms/AdvancedSettings",
  component: AdvancedSettings,
  args: {
    knownPeersSize: 100,
    trustedPeers: [],
    onClearHistory: () => {},
    onChangeKnownPeersSize: () => {},
    onAddTrusted: () => {},
    onRemoveTrusted: () => {},
  },
} satisfies Meta<typeof AdvancedSettings>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Collapsed: Story = {};
export const CollapsedWithContacts: Story = { args: { trustedPeers: [key(1)] } };
export const Open: Story = { args: { defaultOpen: true } };
export const OpenWithContacts: Story = { args: { defaultOpen: true, trustedPeers: [key(1), key(2), key(3)] } };
/** Phone: read-only except players to remember, lock icons on the desktop-only controls. */
export const OpenOnPhone: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { defaultOpen: true, remote: true, trustedPeers: [key(1)] },
};
export const CollapsedOnPhone: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { remote: true },
};
