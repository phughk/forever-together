import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, within } from "storybook/test";
import { ConnectionStatus } from "./ConnectionStatus";

const meta = { title: "Molecules/ConnectionStatus", component: ConnectionStatus, args: { status: "live" } } satisfies Meta<typeof ConnectionStatus>;
export default meta;
type Story = StoryObj<typeof meta>;

/** A healthy connection shows nothing at all. */
export const Live: Story = {
  play: async ({ canvasElement }) => {
    await expect(within(canvasElement).queryByRole("status")).toBeNull();
  },
};
export const Connecting: Story = { args: { status: "connecting" } };
export const Reconnecting: Story = {
  args: { status: "reconnecting" },
  play: async ({ canvasElement }) => {
    await expect(within(canvasElement).getByRole("status")).toHaveTextContent("Reconnecting...");
  },
};
export const NotLinked: Story = { args: { status: "unauthorized" } };
