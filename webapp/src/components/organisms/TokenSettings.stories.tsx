import type { Meta, StoryObj } from "@storybook/react-vite";
import { TokenSettings } from "./TokenSettings";

const meta = { title: "Organisms/TokenSettings", component: TokenSettings, args: { onReset: () => {} } } satisfies Meta<typeof TokenSettings>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Desktop: Story = {};
export const ViewedOnPhone: Story = { args: { remote: true } };
export const ResetFailed: Story = { args: { error: "Could not reset the token. Try again." } };
