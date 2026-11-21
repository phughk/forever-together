import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, within } from "storybook/test";
import { AddonVersionPair } from "./AddonVersionPair";

const meta = { title: "Molecules/AddonVersionPair", component: AddonVersionPair, args: { thisApp: "0.2.0" } } satisfies Meta<typeof AddonVersionPair>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Not installed: only the app's version, labelled "In this app". */
export const AppOnly: Story = {
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await expect(c.getByText("In this app:")).toBeVisible();
    await expect(c.queryByText("In game:")).toBeNull();
  },
};
/** An older version is in the game; the app's is labelled "New". */
export const UpdateAvailable: Story = { args: { inGame: "0.1.0", thisAppLabel: "New" } };
/** Same version: both values stay visible so a reinstall is understood. */
export const SameVersion: Story = {
  args: { inGame: "0.2.0", thisApp: "0.2.0" },
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await expect(c.getByText("In game:")).toBeVisible();
    await expect(c.getByText("This app:")).toBeVisible();
    await expect(c.getAllByText("0.2.0")).toHaveLength(2);
  },
};
export const InstalledNewer: Story = { args: { inGame: "0.3.0" } };
/** After an install: what is in the game now. */
export const Now: Story = { args: { thisAppLabel: "Now" } };
export const LongVersion: Story = { args: { inGame: "10.12.345-beta.2", thisApp: "10.12.346" } };
