import type { Meta, StoryObj } from "@storybook/react-vite";
import { AppHeader } from "./AppHeader";

const meta = {
  title: "Organisms/AppHeader",
  component: AppHeader,
  args: { view: "finder", status: "live", onNavigate: () => {} },
} satisfies Meta<typeof AppHeader>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Finder: Story = {};
export const Settings: Story = { args: { view: "settings" } };
export const Reconnecting: Story = { args: { status: "reconnecting" } };
export const Phone: Story = { globals: { viewport: { value: "mobile1", isRotated: false } } };
