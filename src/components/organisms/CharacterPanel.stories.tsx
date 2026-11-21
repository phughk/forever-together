import type { Meta, StoryObj } from "@storybook/react-vite";
import { MIN, NOW, makeCharacter } from "../../api/fixtures";
import { CharacterPanel } from "./CharacterPanel";

const meta = {
  title: "Organisms/CharacterPanel",
  component: CharacterPanel,
  args: { character: makeCharacter(), now: NOW, onSubmitCode: () => {}, onForget: () => {} },
} satisfies Meta<typeof CharacterPanel>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Understood: Story = {};
export const NoCodeYet: Story = { args: { character: null } };
export const StaleCode: Story = { args: { character: makeCharacter({ createdAt: NOW - 95 * MIN }) } };
export const UnknownZone: Story = { args: { character: makeCharacter({ zone: "Zul'Gurub Outskirts", zoneKnown: false }) } };
export const EmptyZone: Story = { args: { character: makeCharacter({ zone: "", zoneKnown: false }) } };
export const ItemLevelUnknown: Story = { args: { character: makeCharacter({ itemLevel: 0, role: "unknown" }) } };
export const LongNames: Story = {
  args: { character: makeCharacter({ firstName: "Bartholomewmaximilian", surname: "Featherstonehaugh-Cholmondeley" }) },
};
export const SearchRunningLocked: Story = { args: { locked: true } };
export const CodeRefused: Story = {
  args: { error: "This code comes from a different version. Update the addon or the app so both match." },
};
