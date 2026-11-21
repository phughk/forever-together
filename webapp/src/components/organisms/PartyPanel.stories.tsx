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

/** Five slots; the first `n` players (you are the first) have accepted. */
const accepted = (n: number) => makeParty({}, [0, 1, 2, 3, 4].map((i) => ({ confirmed: i < n })));

export const Accepted0: Story = { args: { party: accepted(0) } };
export const Accepted1YouAccepted: Story = { args: { party: accepted(1) } };
export const Accepted2: Story = { args: { party: accepted(2) } };
export const Accepted3: Story = { args: { party: accepted(3) } };
export const Accepted4YouLast: Story = {
  args: { party: makeParty({}, [{ confirmed: false }, { confirmed: true }, { confirmed: true }, { confirmed: true }, { confirmed: true }]) },
};
export const Accepted5: Story = { args: { party: accepted(5) } };
export const LastSecondsToAccept: Story = { args: { party: makeParty({ confirmDeadline: NOW + 2500 }) } };
/** Under 640 px the Accept and Decline buttons are fixed to the bottom of the viewport. */
export const PhoneStickyActions: Story = {
  args: { party: accepted(2) },
  globals: { viewport: { value: "mobile1", isRotated: false } },
};
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
export const FinalPhone: Story = {
  args: FinalWithNames.args,
  globals: { viewport: { value: "mobile1", isRotated: false } },
};
export const DroppedDeclined: Story = { args: { party: makeParty({ phase: "dropped", dropReason: "declined", confirmDeadline: null }), onDismiss: () => {} } };
export const DroppedTimeout: Story = { args: { party: makeParty({ phase: "dropped", dropReason: "timeout", confirmDeadline: null }), onDismiss: () => {} } };
export const DroppedUnreachable: Story = { args: { party: makeParty({ phase: "dropped", dropReason: "unreachable", confirmDeadline: null }), onDismiss: () => {} } };
