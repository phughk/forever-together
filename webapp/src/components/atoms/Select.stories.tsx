import type { Meta, StoryObj } from "@storybook/react-vite";
import { Select } from "./Select";

const meta = {
  title: "Atoms/Select",
  component: Select,
  args: {
    "aria-label": "Address",
    mono: true,
    options: [
      { value: "192.168.1.23", label: "192.168.1.23" },
      { value: "10.0.0.7", label: "10.0.0.7" },
    ],
  },
} satisfies Meta<typeof Select>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Disabled: Story = { args: { disabled: true } };
