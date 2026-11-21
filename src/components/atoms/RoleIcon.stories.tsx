import type { Meta, StoryObj } from "@storybook/react-vite";
import { RoleIcon } from "./RoleIcon";

const meta = { title: "Atoms/RoleIcon", component: RoleIcon } satisfies Meta<typeof RoleIcon>;
export default meta;

export const AllRoles: StoryObj = {
  render: () => (
    <div style={{ display: "flex", gap: "0.5rem" }}>
      <RoleIcon role="tank" />
      <RoleIcon role="healer" />
      <RoleIcon role="damage" />
      <RoleIcon role="unknown" />
    </div>
  ),
};
