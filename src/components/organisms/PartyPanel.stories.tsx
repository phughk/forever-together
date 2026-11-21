import type { Meta, StoryObj } from "@storybook/react-vite";
import { NOW, makeMember, makeParty } from "../../api/fixtures";
import { PartyPanel } from "./PartyPanel";

const meta = {
  title: "Organisms/PartyPanel",
  component: PartyPanel,
  args: { party: makeParty(), now: NOW, onConfirm: () => {}, onDecline: () => {} },
} satisfies Meta<typeof PartyPanel>;
export default meta;
type Story = StoryObj<typeof meta>;

export const AwaitingYourConfirm: Story = {};
export const MemberReadyButNotConfirmed: Story = {
  args: { party: makeParty({}, [{ confirmed: true }, { confirmed: true }, { confirmed: true }, { confirmed: true }, { confirmed: false, ready: true }]) },
};
export const YouConfirmedOthersPending: Story = {
  args: { party: makeParty({}, [{ confirmed: true }, { confirmed: true }, { ready: false }, { ready: false }, { ready: false }]) },
};
export const LastSecondsToConfirm: Story = { args: { party: makeParty({ confirmDeadline: NOW + 2500 }) } };
export const FinalWithNames: Story = {
  args: {
    party: {
      ...makeParty({ phase: "final", confirmDeadline: null }),
      members: [0, 1, 2, 3, 4].map((i) => makeMember(i, { confirmed: true })),
    },
  },
};
export const FinalWithLongNames: Story = {
  args: {
    party: {
      ...makeParty({ phase: "final", confirmDeadline: null }),
      members: [0, 1, 2, 3, 4].map((i) =>
        makeMember(i, i === 2 ? { confirmed: true, firstName: "Bartholomewmaximilian", surname: "Featherstonehaugh-Cholmondeley" } : { confirmed: true }),
      ),
    },
  },
};
export const DroppedDeclined: Story = { args: { party: makeParty({ phase: "dropped", dropReason: "declined", confirmDeadline: null }), onDismiss: () => {} } };
export const DroppedTimeout: Story = { args: { party: makeParty({ phase: "dropped", dropReason: "timeout", confirmDeadline: null }) } };
export const DroppedUnreachable: Story = { args: { party: makeParty({ phase: "dropped", dropReason: "unreachable", confirmDeadline: null }) } };
