import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, fn, userEvent, within } from "storybook/test";
import { MIN, NOW, makeCharacter } from "../../api/fixtures";
import { CharacterStrip } from "./CharacterStrip";

const meta = {
  title: "Organisms/CharacterStrip",
  component: CharacterStrip,
  args: { character: makeCharacter(), now: NOW, onSubmitCode: () => {}, onForget: fn() },
} satisfies Meta<typeof CharacterStrip>;
export default meta;
type Story = StoryObj<typeof meta>;

/** One line: name, class, level, role, item level. Edit is the only control. */
export const Closed: Story = {};
/** The stale badge and one short line stay inside the strip. */
export const OutOfDate: Story = { args: { character: makeCharacter({ createdAt: NOW - 95 * MIN }) } };
export const ZoneNotRecognised: Story = { args: { character: makeCharacter({ zone: "Zul'Gurub Outskirts", zoneKnown: false }) } };
export const LongNames: Story = {
  args: { character: makeCharacter({ firstName: "Bartholomewmaximilianx", surname: "Featherstonehaughcholmo" }) },
};
export const LongNamesOutOfDate: Story = {
  args: { character: makeCharacter({ firstName: "Bartholomewmaximilianx", surname: "Featherstonehaughcholmo", createdAt: NOW - 95 * MIN }) },
};
export const NarrowPhone: Story = { decorators: [(Story) => <div style={{ width: 320 }}>{Story()}</div>] };
export const Open: Story = { args: { defaultOpen: true } };
export const OpenOutOfDate: Story = { args: { defaultOpen: true, character: makeCharacter({ createdAt: NOW - 95 * MIN }) } };
export const CodeRefused: Story = {
  args: { error: "This code comes from a different version. Update the addon or the app so both match." },
};

/** Edit opens update and remove inline; Close hides them again. */
export const EditOpensAndCloses: Story = {
  play: async ({ canvasElement, args }) => {
    const c = within(canvasElement);
    await expect(c.queryByRole("button", { name: "Remove character" })).toBeNull();
    await userEvent.click(c.getByRole("button", { name: "Edit" }));
    await expect(c.getByRole("button", { name: "Update character" })).toBeVisible();
    await userEvent.click(c.getByRole("button", { name: "Remove character" }));
    await expect(args.onForget).toHaveBeenCalled();
    await userEvent.click(c.getByRole("button", { name: "Close" }));
    await expect(c.queryByRole("button", { name: "Update character" })).toBeNull();
  },
};
