import type { Meta, StoryObj } from "@storybook/react-vite";
import { InlineForm } from "./InlineForm";

const meta = {
  title: "Molecules/InlineForm",
  component: InlineForm,
  args: { label: "Add a trusted peer", submitLabel: "Add", onSubmit: () => {}, placeholder: "64 hex characters", mono: true },
} satisfies Meta<typeof InlineForm>;
export default meta;
type Story = StoryObj<typeof meta>;

export const TrustedPeer: Story = {};
export const Number: Story = {
  args: { label: "Known peers cache size", submitLabel: "Save", type: "number", initialValue: "100", mono: false, placeholder: undefined },
};
export const Disabled: Story = { args: { disabled: true } };
