import type { Meta, StoryObj } from "@storybook/react-vite";
import { dungeonsFor } from "../../api/fixtures";
import { DungeonPicker } from "./DungeonPicker";

const rec = dungeonsFor(24, "Westfall");

const meta = {
  title: "Organisms/DungeonPicker",
  component: DungeonPicker,
  args: {
    dungeons: rec.dungeons,
    recommended: rec.recommended,
    selected: [],
    zone: "Westfall",
    onSelectionChange: () => {},
    onStart: () => {},
  },
} satisfies Meta<typeof DungeonPicker>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Recommended: Story = {};
export const OneSelected: Story = { args: { selected: ["the-deadmines"] } };
export const SeveralSelected: Story = { args: { selected: ["the-deadmines", "wailing-caverns", "shadowfang-keep"] } };
export const NoRecommendations: Story = { args: { recommended: [], zone: "Orgrimmar" } };
export const NoCharacter: Story = { args: { needsCharacter: true, recommended: [] } };
export const SearchRunning: Story = { args: { searching: true, selected: ["the-deadmines"] } };
