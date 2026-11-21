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

export const ReadyNotConfirmed: Story = {};
export const Confirmed: Story = { args: { member: makeMember(1, { confirmed: true, firstName: null, surname: null }) } };
export const WaitingForReady: Story = { args: { member: makeMember(2, { ready: false, firstName: null, surname: null }) } };
export const Declined: Story = { args: { member: makeMember(3, { declined: true, firstName: null, surname: null }) } };
export const You: Story = { args: { member: makeMember(0, { confirmed: true, firstName: null, surname: null }) } };
export const NameRevealed: Story = { args: { member: makeMember(1, { confirmed: true }), revealName: true } };
export const LongNameRevealed: Story = {
  args: {
    member: makeMember(1, { confirmed: true, firstName: "Bartholomewmaximilian", surname: "Featherstonehaugh-Cholmondeley" }),
    revealName: true,
  },
};
