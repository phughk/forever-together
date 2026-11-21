import type { Meta, StoryObj } from "@storybook/react-vite";
import { PeerRow } from "./PeerRow";

const meta = {
  title: "Molecules/PeerRow",
  component: PeerRow,
  args: { peerKey: "6b1f0c9d2a7e4d38b5c1a09e7f23d46c8b0a15e9d374f2c6a81b0d95e7c3f402", onRemove: () => {} },
} satisfies Meta<typeof PeerRow>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Removable: Story = {};
export const ReadOnly: Story = { args: { onRemove: undefined } };
