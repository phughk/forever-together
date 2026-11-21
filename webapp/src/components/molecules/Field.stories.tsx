import type { Meta, StoryObj } from "@storybook/react-vite";
import { Badge } from "../atoms/Badge";
import { Field } from "./Field";

const meta = { title: "Molecules/Field", component: Field, args: { label: "Level", children: "24" } } satisfies Meta<typeof Field>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const WithBadge: Story = { args: { label: "Faction", children: <Badge tone="alliance">Alliance</Badge> } };
export const LongValue: Story = {
  args: { label: "Zone", children: "A zone name that is much longer than anyone would expect to see in the game client" },
};
