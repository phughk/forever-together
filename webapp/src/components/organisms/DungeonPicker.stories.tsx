import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, fn, userEvent, within } from "storybook/test";
import { DUNGEON_TABLE, dungeonsFor } from "../../api/fixtures";
import { DungeonPicker } from "./DungeonPicker";

const rec = dungeonsFor(24, "Westfall");

/** 30 dungeons: the table repeated with new slugs, to prove the list scrolls inside its own region. */
const many = Array.from({ length: 30 }, (_, i) => {
  const base = DUNGEON_TABLE[i % DUNGEON_TABLE.length]!;
  return { ...base, slug: `${base.slug}-${i}`, name: i < DUNGEON_TABLE.length ? base.name : `${base.name} (${i})` };
});

const meta = {
  title: "Organisms/DungeonPicker",
  component: DungeonPicker,
  args: {
    dungeons: rec.dungeons,
    recommended: rec.recommended,
    zone: "Westfall",
    onStart: fn(),
  },
} satisfies Meta<typeof DungeonPicker>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Recommended: Story = {};
export const OneDungeon: Story = { args: { dungeons: [DUNGEON_TABLE[2]!], recommended: ["the-deadmines"] } };
export const ThirtyDungeons: Story = {
  args: { dungeons: many, recommended: many.slice(0, 4).map((d) => d.slug) },
};
export const NoRecommendations: Story = { args: { recommended: [], zone: "Orgrimmar" } };
/** No character yet: only the greyed step, no list. */
export const NoCharacter: Story = { args: { needsCharacter: true, recommended: [] } };
export const SearchRunning: Story = { args: { searching: true } };
export const Busy: Story = { args: { busy: true } };
export const CombineOn: Story = { args: { defaultCombine: true } };
export const CombineWithSelection: Story = { args: { defaultCombine: true, selected: ["the-deadmines", "wailing-caverns"] } };
export const CombineThirty: Story = {
  args: { dungeons: many, recommended: many.slice(0, 4).map((d) => d.slug), defaultCombine: true, selected: [many[0]!.slug] },
};
export const NarrowPhone: Story = {
  decorators: [(Story) => <div style={{ width: 320 }}>{Story()}</div>],
};

/** Tapping a row starts the search for that one dungeon. */
export const TapStarts: Story = {
  play: async ({ canvasElement, args }) => {
    await userEvent.click(within(canvasElement).getByRole("button", { name: "Search The Deadmines" }));
    await expect(args.onStart).toHaveBeenCalledWith(["the-deadmines"]);
  },
};

/** With "Combine dungeons" on, ticks collect and "Search N" starts them together. */
export const CombineFlow: Story = {
  play: async ({ canvasElement, args }) => {
    const c = within(canvasElement);
    await userEvent.click(c.getByRole("switch", { name: "Combine dungeons" }));
    await expect(c.getByRole("button", { name: "Search 0" })).toBeDisabled();
    await userEvent.click(c.getByRole("checkbox", { name: "Include The Deadmines" }));
    await userEvent.click(c.getByRole("checkbox", { name: "Include Wailing Caverns" }));
    await userEvent.click(c.getByRole("button", { name: "Search 2" }));
    await expect(args.onStart).toHaveBeenCalledWith(["the-deadmines", "wailing-caverns"]);
  },
};
