import type { Meta, StoryObj } from "@storybook/react-vite";
import { HistorySettings } from "./HistorySettings";

const meta = { title: "Organisms/HistorySettings", component: HistorySettings, args: { onClear: () => {} } } satisfies Meta<typeof HistorySettings>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Desktop: Story = {};
export const ViewedOnPhone: Story = { args: { remote: true } };
export const ClearFailed: Story = { args: { error: "The search history could not be cleared. Try again." } };
