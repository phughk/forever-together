import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, within } from "storybook/test";
import { AppHeader } from "./AppHeader";

const meta = {
  title: "Organisms/AppHeader",
  component: AppHeader,
  args: { view: "finder", status: "live", onNavigate: () => {} },
  parameters: { layout: "padded" },
} satisfies Meta<typeof AppHeader>;
export default meta;
type Story = StoryObj<typeof meta>;

const phone = { viewport: { value: "mobile1", isRotated: false } };

/** Fine: only the gear. No name, no "Online". */
export const Finder: Story = {
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await expect(c.getByRole("button", { name: "Settings" })).toBeVisible();
    await expect(c.queryByRole("status")).toBeNull();
  },
};
/** On Settings: Back, nothing else. */
export const Settings: Story = {
  args: { view: "settings" },
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await expect(c.getByRole("button", { name: "Back" })).toBeVisible();
    await expect(c.queryByRole("button", { name: "Settings" })).toBeNull();
  },
};
export const Connecting: Story = { args: { status: "connecting" } };
export const Reconnecting: Story = { args: { status: "reconnecting" } };
export const NotLinked: Story = { args: { status: "unauthorized" } };
export const SettingsReconnecting: Story = { args: { view: "settings", status: "reconnecting" } };
export const PhoneFinder: Story = { globals: phone };
export const PhoneSettings: Story = { args: { view: "settings" }, globals: phone };
export const PhoneConnecting: Story = { args: { status: "connecting" }, globals: phone };
export const PhoneReconnecting: Story = { args: { status: "reconnecting" }, globals: phone };
export const PhoneNotLinked: Story = { args: { status: "unauthorized" }, globals: phone };
