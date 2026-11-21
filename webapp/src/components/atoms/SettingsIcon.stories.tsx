import type { Meta, StoryObj } from "@storybook/react-vite";
import { SettingsIcon } from "./SettingsIcon";

const meta = { title: "Atoms/SettingsIcon", component: SettingsIcon, args: { name: "lock" } } satisfies Meta<typeof SettingsIcon>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Lock: Story = { args: { label: "Change this on your PC" } };
export const All: Story = {
  render: () => (
    <div style={{ display: "flex", gap: 16 }}>
      <SettingsIcon name="lock" />
      <SettingsIcon name="monitor" />
      <SettingsIcon name="switch" />
      <SettingsIcon name="qr" />
      <SettingsIcon name="flag" />
    </div>
  ),
};
export const Large: Story = { args: { name: "qr", size: 40 } };
