import type { Meta, StoryObj } from "@storybook/react-vite";
import { TokenSettings } from "./TokenSettings";

const meta = { title: "Organisms/TokenSettings (Sign out all phones)", component: TokenSettings, args: { onReset: () => {} } } satisfies Meta<typeof TokenSettings>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Desktop: Story = {};
export const ViewedOnPhone: Story = { args: { remote: true } };
export const SignOutFailed: Story = { args: { error: "Could not sign out the phones. Try again." } };
