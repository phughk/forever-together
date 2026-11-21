import type { Meta, StoryObj } from "@storybook/react-vite";
import { StageStep } from "./StageStep";

const meta = {
  title: "Molecules/StageStep",
  component: StageStep,
  args: { number: 2, title: "Wider item level", rule: "Item level within 30%", state: "current" },
  decorators: [(Story) => <ol style={{ listStyle: "none", padding: 0, margin: 0 }}><Story /></ol>],
} satisfies Meta<typeof StageStep>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Current: Story = {};
export const Done: Story = { args: { state: "done", number: 0, title: "Strict" } };
export const Upcoming: Story = { args: { state: "upcoming", startsInS: 42, number: 3 } };
