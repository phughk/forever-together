import type { Meta, StoryObj } from "@storybook/react-vite";
import { DUNGEON_TABLE } from "../../api/fixtures";
import { DungeonOption } from "./DungeonOption";

const deadmines = DUNGEON_TABLE[2]!;
const maraudon = DUNGEON_TABLE[11]!;

const meta = {
  title: "Molecules/DungeonOption",
  component: DungeonOption,
  args: { dungeon: deadmines, selected: false, onToggle: () => {} },
} satisfies Meta<typeof DungeonOption>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Selected: Story = { args: { selected: true } };
export const InCurrentZone: Story = { args: { inZone: true } };
export const ComingSoon: Story = { args: { dungeon: maraudon } };
export const SearchRunning: Story = { args: { selected: true, disabled: true } };
export const LongName: Story = {
  args: { dungeon: { ...deadmines, name: "The Extremely Long Named Dungeon of Many Winding Corridors and Forgotten Halls" } },
};
