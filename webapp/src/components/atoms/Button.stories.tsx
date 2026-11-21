import type { Meta, StoryObj } from "@storybook/react-vite";
import { Button } from "./Button";

const meta = {
  title: "Atoms/Button",
  component: Button,
  args: { children: "Start search" },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Primary: Story = { args: { variant: "primary" } };
export const Danger: Story = { args: { variant: "danger", children: "Sign out all phones" } };
export const Ghost: Story = { args: { variant: "ghost", children: "Cancel" } };
export const Small: Story = { args: { size: "small", children: "Remove" } };
export const Block: Story = { args: { block: true, variant: "primary" } };
export const Disabled: Story = { args: { disabled: true } };
