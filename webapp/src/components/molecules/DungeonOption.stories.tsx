import type { Meta, StoryObj } from "@storybook/react-vite";
import { DUNGEON_TABLE } from "../../api/fixtures";
import { DungeonOption } from "./DungeonOption";

const deadmines = DUNGEON_TABLE[2]!;

const meta = {
  title: "Molecules/DungeonOption",
  component: DungeonOption,
  args: { dungeon: deadmines, onStart: () => {}, onToggle: () => {} },
} satisfies Meta<typeof DungeonOption>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Single mode: the whole row starts the search. */
export const Default: Story = {};
export const InCurrentZone: Story = { args: { inZone: true } };
/** A search is running or a request is in flight. */
export const Busy: Story = { args: { disabled: true } };
export const LongName: Story = {
  args: { dungeon: { ...deadmines, name: "The Extremely Long Named Dungeon of Many Winding Corridors and Forgotten Halls" } },
};
/** Combine mode, not ticked. */
export const CombineUnticked: Story = { args: { multi: true, selected: false } };
export const CombineTicked: Story = { args: { multi: true, selected: true } };
export const CombineDisabled: Story = { args: { multi: true, selected: true, disabled: true } };
