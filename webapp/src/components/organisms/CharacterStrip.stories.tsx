import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, fn, userEvent, within } from "storybook/test";
import { MIN, NOW, makeCharacter } from "../../api/fixtures";
import { CharacterStrip } from "./CharacterStrip";

const meta = {
  title: "Organisms/CharacterStrip",
  component: CharacterStrip,
  args: { character: makeCharacter(), now: NOW, onSubmitCode: fn(), onForget: fn() },
} satisfies Meta<typeof CharacterStrip>;
export default meta;
type Story = StoryObj<typeof meta>;

const phone = { globals: { viewport: { value: "mobile1", isRotated: false } } } as const;

/** Fresh code: the summary and a visible, ordinary "Change character" button. */
export const IdleFresh: Story = {};
/** Out of date: the badge says so and the same control becomes the highlighted "Update". */
export const IdleOutOfDate: Story = { args: { character: makeCharacter({ createdAt: NOW - 95 * MIN }) } };
/** The paste form is open inline, with Cancel. */
export const FormOpen: Story = { args: { defaultOpen: true } };
export const FormOpenOutOfDate: Story = { args: { defaultOpen: true, character: makeCharacter({ createdAt: NOW - 95 * MIN }) } };
/** The app refused the pasted code: the reason shows inline and the form stays open. */
export const SubmitError: Story = {
  args: { error: "This code comes from a different version. Update the addon or the app so both match." },
};
export const Busy: Story = { args: { defaultOpen: true, busy: true } };
/** Searching: the control is disabled and a short reason is shown. */
export const SearchingDisabled: Story = { args: { lockedReason: "Stop the search to change character." } };
export const InParty: Story = { args: { lockedReason: "Leave the party to change character." } };
export const ZoneNotRecognised: Story = { args: { character: makeCharacter({ zone: "Zul'Gurub Outskirts", zoneKnown: false }) } };
export const LongNames: Story = {
  args: { character: makeCharacter({ firstName: "Bartholomewmaximilianx", surname: "Featherstonehaughcholmo" }) },
};
export const LongNamesOutOfDate: Story = {
  args: { character: makeCharacter({ firstName: "Bartholomewmaximilianx", surname: "Featherstonehaughcholmo", createdAt: NOW - 95 * MIN }) },
};
export const NarrowPhone: Story = { decorators: [(Story) => <div style={{ width: 320 }}>{Story()}</div>] };
/** A phone: the button is a full-width 44 px tap target under the summary. */
export const PhoneWidth: Story = { ...phone, args: { defaultOpen: true, character: makeCharacter({ createdAt: NOW - 95 * MIN }) } };

/** The button opens the form; Cancel and Escape close it; submitting passes the pasted code on. */
export const ChangeOpensForm: Story = {
  play: async ({ canvasElement, args }) => {
    const c = within(canvasElement);
    await expect(c.queryByLabelText("Character code from the addon")).toBeNull();
    await userEvent.click(c.getByRole("button", { name: "Change character" }));
    const box = c.getByLabelText("Character code from the addon");
    await expect(box).toHaveFocus();
    await userEvent.keyboard("{Escape}");
    await expect(c.queryByLabelText("Character code from the addon")).toBeNull();
    await userEvent.click(c.getByRole("button", { name: "Change character" }));
    await userEvent.click(c.getByRole("button", { name: "Cancel" }));
    await expect(c.queryByLabelText("Character code from the addon")).toBeNull();
    await userEvent.click(c.getByRole("button", { name: "Change character" }));
    await userEvent.type(c.getByLabelText("Character code from the addon"), "WFT1:ABC");
    await userEvent.click(c.getByRole("button", { name: "Use this code" }));
    await expect(args.onSubmitCode).toHaveBeenCalledWith("WFT1:ABC");
  },
};

/** Remove stays available, but behind Details and secondary. */
export const RemoveIsSecondary: Story = {
  play: async ({ canvasElement, args }) => {
    const c = within(canvasElement);
    await userEvent.click(c.getByText("Details"));
    await userEvent.click(c.getByRole("button", { name: "Remove character" }));
    await expect(args.onForget).toHaveBeenCalled();
  },
};

/** While searching the button is disabled. */
export const DisabledWhileSearching: Story = {
  args: { lockedReason: "Stop the search to change character." },
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await expect(c.getByRole("button", { name: "Change character" })).toBeDisabled();
    await expect(c.getByText("Stop the search to change character.")).toBeVisible();
  },
};
