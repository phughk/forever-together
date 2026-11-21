import type { Meta, StoryObj } from "@storybook/react-vite";
import { Switch } from "../atoms/Toggle";
import { SettingRow } from "./SettingRow";

const meta = {
  title: "Molecules/SettingRow",
  component: SettingRow,
  args: {
    title: "LAN mode",
    description: "Lets phones on your network open the app.",
    children: <Switch aria-label="LAN mode" />,
  },
} satisfies Meta<typeof SettingRow>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const WithHint: Story = { args: { description: undefined, hint: "Phones must be on the same Wi-Fi as this computer." } };
export const NoDescription: Story = { args: { description: undefined } };
