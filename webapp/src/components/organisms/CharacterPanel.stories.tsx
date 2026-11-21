import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";
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
/** First run: steps 1 to 3. Step 1 is plain text because ADDON_URL is not set yet; with a URL it becomes a link. */
export const FirstRun: Story = { args: { character: null } };
export const FirstRunWithAddonLink: Story = { args: { character: null, addonUrl: "https://example.invalid/addon" } };
export const FirstRunCodeRefused: Story = {
  args: { character: null, error: "This code comes from a different version. Update the addon or the app so both match." },
};
export const FirstRunBusy: Story = { args: { character: null, busy: true } };
export const FirstRunNarrowPhone: Story = {
  args: { character: null },
  decorators: [(Story) => <div style={{ width: 320 }}>{Story()}</div>],
};
export const OutOfDate: Story = { args: { character: makeCharacter({ createdAt: NOW - 40 * MIN }) } };
export const ZoneNotRecognised: Story = { args: { character: makeCharacter({ zone: "Zul'Gurub Outskirts", zoneKnown: false }) } };
export const EmptyZone: Story = { args: { character: makeCharacter({ zone: "", zoneKnown: false }) } };
export const OutOfDateAndZoneNotRecognised: Story = {
  args: { character: makeCharacter({ createdAt: NOW - 95 * MIN, zoneKnown: false }) },
};
export const DetailsOpen: Story = {
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await userEvent.click(c.getByText("Details"));
    await expect(c.getByText("Updated")).toBeVisible();
  },
};
export const ItemLevelUnknown: Story = { args: { character: makeCharacter({ itemLevel: 0, role: "unknown" }) } };
export const RolePlayRealm: Story = {
  args: { character: makeCharacter({ realm: "18", realmName: "RP-PvP", faction: "horde" }) },
};
export const UnknownRealmId: Story = {
  args: { character: makeCharacter({ realm: "27", realmName: "realm 27", region: 9, regionName: "region 9" }) },
};
export const LongNames: Story = {
  args: { character: makeCharacter({ firstName: "Bartholomewmaximilian", surname: "Featherstonehaugh-Cholmondeley" }) },
};
export const NarrowPhone: Story = { decorators: [(Story) => <div style={{ width: 320 }}>{Story()}</div>] };
export const CodeRefused: Story = {
  args: { error: "This code comes from a different version. Update the addon or the app so both match." },
};
