import type { Meta, StoryObj } from "@storybook/react-vite";
import { LanSettings } from "./LanSettings";

const URL = "http://192.168.1.23:41873/#token=3f9a1c0be27d44a8b6d15e90c7a2f8e14b0d9c63a5e7f1280b4d6c9a1e3f5072";

const meta = {
  title: "Organisms/LanSettings",
  component: LanSettings,
  args: { lanMode: false, onChange: () => {}, pairing: { kind: "off" } },
} satisfies Meta<typeof LanSettings>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Off: Story = {};
export const OnWithPairingQr: Story = { args: { lanMode: true, pairing: { kind: "ready", url: URL } } };
export const OnLoading: Story = { args: { lanMode: true, pairing: { kind: "loading" } } };
export const ViewedOnPhone: Story = { args: { lanMode: true, remote: true, pairing: { kind: "forbidden" } } };
export const PairingError: Story = { args: { lanMode: true, pairing: { kind: "error", message: "No private network address found." } } };
