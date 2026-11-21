import type { Meta, StoryObj } from "@storybook/react-vite";
import { makeCharacter } from "../../api/fixtures";
import { CharacterSummary } from "./CharacterSummary";

const meta = {
  title: "Molecules/CharacterSummary",
  component: CharacterSummary,
  args: { character: makeCharacter() },
} satisfies Meta<typeof CharacterSummary>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const OutOfDate: Story = { args: { stale: true } };
export const ZoneNotRecognised: Story = { args: { character: makeCharacter({ zone: "Zul'Gurub Outskirts", zoneKnown: false }) } };
export const AllBadges: Story = { args: { stale: true, character: makeCharacter({ zoneKnown: false }) } };
export const ItemLevelUnknown: Story = { args: { character: makeCharacter({ itemLevel: 0, role: "unknown" }) } };
export const LongNames: Story = {
  args: { character: makeCharacter({ firstName: "Bartholomewmaximilian", surname: "Featherstonehaugh-Cholmondeley" }) },
};
export const NarrowPhone: Story = { decorators: [(Story) => <div style={{ width: 320 }}>{Story()}</div>] };
