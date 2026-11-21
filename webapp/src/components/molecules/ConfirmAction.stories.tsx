import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";
import { ConfirmAction } from "./ConfirmAction";

const meta = {
  title: "Molecules/ConfirmAction",
  component: ConfirmAction,
  args: {
    label: "Sign out",
    warning: "Phones must scan the QR again.",
    confirmLabel: "Yes, sign out",
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
    await userEvent.click(canvas.getByRole("button", { name: "Sign out" }));
    await expect(canvas.getByRole("button", { name: "Yes, sign out" })).toBeVisible();
  },
};
