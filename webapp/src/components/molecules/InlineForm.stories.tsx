import type { Meta, StoryObj } from "@storybook/react-vite";
import { InlineForm } from "./InlineForm";

const meta = {
  title: "Molecules/InlineForm",
  component: InlineForm,
  args: { label: "Add a backup contact", submitLabel: "Add", onSubmit: () => {}, placeholder: "64 hex characters", mono: true },
} satisfies Meta<typeof InlineForm>;
export default meta;
type Story = StoryObj<typeof meta>;

export const BackupContact: Story = {};
export const Number: Story = {
  args: { label: "Players to remember", submitLabel: "Save", type: "number", initialValue: "100", mono: false, placeholder: undefined },
};
export const Disabled: Story = { args: { disabled: true } };
