import type { Meta, StoryObj } from "@storybook/react-vite";
import { makeMember } from "../../api/fixtures";
import { MemberRow } from "./MemberRow";

const meta = {
  title: "Molecules/MemberRow",
  component: MemberRow,
  args: { member: makeMember(1, { firstName: null, surname: null }), revealName: false },
} satisfies Meta<typeof MemberRow>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Waiting: Story = { args: { member: makeMember(1, { confirmed: false, firstName: null, surname: null }) } };
export const Accepted: Story = { args: { member: makeMember(1, { confirmed: true, firstName: null, surname: null }) } };
export const YouWaiting: Story = { args: { member: makeMember(0, { confirmed: false, firstName: null, surname: null }) } };
export const YouAccepted: Story = { args: { member: makeMember(0, { confirmed: true, firstName: null, surname: null }) } };
export const NameWithCopyButton: Story = { args: { member: makeMember(1, { confirmed: true }), revealName: true } };
export const YouInFinalParty: Story = { args: { member: makeMember(0, { confirmed: true }), revealName: true } };
export const LongNameRevealed: Story = {
  args: {
    member: makeMember(1, { confirmed: true, firstName: "Bartholomewmaximilian", surname: "Featherstonehaugh-Cholmondeley" }),
    revealName: true,
  },
};
export const Phone: Story = {
  args: { member: makeMember(1, { confirmed: true }), revealName: true },
  globals: { viewport: { value: "mobile1", isRotated: false } },
};
