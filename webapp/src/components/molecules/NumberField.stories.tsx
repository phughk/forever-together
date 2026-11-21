import type { Meta, StoryObj } from "@storybook/react-vite";
import { NumberField } from "./NumberField";

const meta = { title: "Molecules/NumberField", component: NumberField, args: { label: "Players to remember", value: 100, onSave: () => {} } } satisfies Meta<typeof NumberField>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Large: Story = { args: { value: 1000 } };
export const Disabled: Story = { args: { disabled: true } };
