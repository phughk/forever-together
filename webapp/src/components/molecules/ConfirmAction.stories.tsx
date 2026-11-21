import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";
import { ConfirmAction } from "./ConfirmAction";

const meta = {
  title: "Molecules/ConfirmAction",
  component: ConfirmAction,
  args: {
    label: "Reset token",
    warning: "Every paired phone will be signed out and must scan the pairing QR again.",
    confirmLabel: "Yes, reset",
    onConfirm: () => {},
  },
} satisfies Meta<typeof ConfirmAction>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Idle: Story = {};
export const Disabled: Story = { args: { disabled: true } };
export const Asking: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(canvas.getByRole("button", { name: "Reset token" }));
    await expect(canvas.getByRole("button", { name: "Yes, reset" })).toBeVisible();
  },
};
