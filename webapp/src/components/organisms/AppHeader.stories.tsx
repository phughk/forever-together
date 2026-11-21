import type { Meta, StoryObj } from "@storybook/react-vite";
import { AppHeader } from "./AppHeader";

const meta = {
  title: "Organisms/AppHeader",
  component: AppHeader,
  args: { view: "finder", status: "live", onNavigate: () => {} },
} satisfies Meta<typeof AppHeader>;
export default meta;
type Story = StoryObj<typeof meta>;

const phone = { viewport: { value: "mobile1", isRotated: false } };

export const Finder: Story = {};
export const Settings: Story = { args: { view: "settings" } };
export const Connecting: Story = { args: { status: "connecting" } };
export const Offline: Story = { args: { status: "reconnecting" } };
export const NotPaired: Story = { args: { status: "unauthorized" } };
export const PhoneOnline: Story = { globals: phone };
export const PhoneSettings: Story = { args: { view: "settings" }, globals: phone };
export const PhoneConnecting: Story = { args: { status: "connecting" }, globals: phone };
export const PhoneOffline: Story = { args: { status: "reconnecting" }, globals: phone };
export const PhoneNotPaired: Story = { args: { status: "unauthorized" }, globals: phone };
