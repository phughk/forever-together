import type { Meta, StoryObj } from "@storybook/react-vite";
import { StatusDot } from "./StatusDot";

const meta = { title: "Atoms/StatusDot", component: StatusDot } satisfies Meta<typeof StatusDot>;
export default meta;

export const AllTones: StoryObj = {
  render: () => (
    <div style={{ display: "flex", gap: "0.75rem" }}>
      <StatusDot />
      <StatusDot tone="good" />
      <StatusDot tone="warn" />
      <StatusDot tone="bad" />
    </div>
  ),
};
