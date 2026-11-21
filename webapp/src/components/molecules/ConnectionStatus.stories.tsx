import type { Meta, StoryObj } from "@storybook/react-vite";
import { ConnectionStatus } from "./ConnectionStatus";

const meta = { title: "Molecules/ConnectionStatus", component: ConnectionStatus, args: { status: "live" } } satisfies Meta<typeof ConnectionStatus>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Live: Story = {};
export const Connecting: Story = { args: { status: "connecting" } };
export const Reconnecting: Story = { args: { status: "reconnecting" } };
export const Unauthorized: Story = { args: { status: "unauthorized" } };
