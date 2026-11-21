import type { Meta, StoryObj } from "@storybook/react-vite";
import { Countdown } from "./Countdown";

const meta = {
  title: "Molecules/Countdown",
  component: Countdown,
  args: { label: "Time to confirm", remainingMs: 11_000, totalMs: 15_000 },
} satisfies Meta<typeof Countdown>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Plenty: Story = {};
export const Urgent: Story = { args: { remainingMs: 3000 } };
export const Expired: Story = { args: { remainingMs: 0 } };
