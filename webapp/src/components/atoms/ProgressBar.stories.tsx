import type { Meta, StoryObj } from "@storybook/react-vite";
import { ProgressBar } from "./ProgressBar";

const meta = {
  title: "Atoms/ProgressBar",
  component: ProgressBar,
  args: { value: 0.4, label: "Progress" },
} satisfies Meta<typeof ProgressBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Empty: Story = { args: { value: 0 } };
export const Full: Story = { args: { value: 1 } };
export const Urgent: Story = { args: { value: 0.15, urgent: true } };
export const Thick: Story = { args: { value: 0.4, size: "thick" } };
export const ThickUrgent: Story = { args: { value: 0.92, size: "thick", urgent: true } };
