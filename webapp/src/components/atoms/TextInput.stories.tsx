import type { Meta, StoryObj } from "@storybook/react-vite";
import { TextArea, TextInput } from "./TextInput";

const meta = {
  title: "Atoms/TextInput",
  component: TextInput,
  args: { placeholder: "Type here", "aria-label": "Example" },
} satisfies Meta<typeof TextInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Monospace: Story = { args: { mono: true, defaultValue: "1a2b3c4d5e6f" } };
export const Invalid: Story = { args: { invalid: true, defaultValue: "bad value" } };
export const Disabled: Story = { args: { disabled: true } };
export const Multiline: StoryObj = {
  render: () => <TextArea mono placeholder="Paste the WFT1: code" aria-label="Code" />,
};
