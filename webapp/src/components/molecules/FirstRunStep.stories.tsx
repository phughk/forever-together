import type { Meta, StoryObj } from "@storybook/react-vite";
import { CopyChip } from "../atoms/CopyChip";
import { FirstRunStep } from "./FirstRunStep";

const meta = {
  title: "Molecules/FirstRunStep",
  component: FirstRunStep,
  args: { n: 1, title: "Get the addon" },
  render: (args) => (
    <ol className="ft-firststeps">
      <FirstRunStep {...args} />
    </ol>
  ),
} satisfies Meta<typeof FirstRunStep>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Plain: Story = {};
export const WithContent: Story = {
  args: {
    n: 2,
    title: (
      <>
        Type <CopyChip text="/wowft" />, copy
      </>
    ),
  },
};
export const Pending: Story = { args: { n: 4, title: "Pick a dungeon", pending: true } };
