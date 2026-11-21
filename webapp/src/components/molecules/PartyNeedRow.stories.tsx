import type { Meta, StoryObj } from "@storybook/react-vite";
import { PartyNeedRow } from "./PartyNeedRow";

const meta = {
  title: "Molecules/PartyNeedRow",
  component: PartyNeedRow,
  parameters: {
    docs: {
      description: {
        component:
          "The three roles against what a party needs (1 tank, 1 healer, 3 damage), as seen / needed, in three equal tiles that fill the card width (role icon, large count, role name). Your own role is pre-filled. Grey means not seen from here, never red, so none seen does not read as none exist. `~` marks rough counts.",
      },
    },
  },
  args: { seen: { tanks: 2, healers: 2, damage: 9 }, ownRole: "damage", rough: false },
} satisfies Meta<typeof PartyNeedRow>;
export default meta;
type Story = StoryObj<typeof meta>;

export const EnoughOfEverything: Story = {};
export const NoTankSeen: Story = { args: { seen: { tanks: 0, healers: 2, damage: 6 } } };
export const YouAreTheTank: Story = { args: { ownRole: "tank", seen: { tanks: 0, healers: 2, damage: 6 } } };
export const NothingSeen: Story = { args: { ownRole: "healer", seen: { tanks: 0, healers: 0, damage: 0 } } };
export const ShortOnDamage: Story = { args: { seen: { tanks: 1, healers: 2, damage: 1 } } };
export const RoughCounts: Story = { args: { rough: true, seen: { tanks: 8, healers: 9, damage: 47 } } };
export const UnknownOwnRole: Story = { args: { ownRole: null } };
export const PhoneWidth: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { rough: true, seen: { tanks: 0, healers: 12, damage: 52 } },
};
