import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";
import { HistorySettings } from "./HistorySettings";

const meta = { title: "Organisms/HistorySettings", component: HistorySettings, args: { onClear: () => {} } } satisfies Meta<typeof HistorySettings>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Desktop: Story = {};
export const Confirming: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(canvas.getByRole("button", { name: "Clear" }));
    await expect(canvas.getByRole("button", { name: "Yes, clear it" })).toBeVisible();
  },
};
export const ViewedOnPhone: Story = { args: { remote: true } };
export const ClearFailed: Story = { args: { error: "The search history could not be cleared. Try again." } };
