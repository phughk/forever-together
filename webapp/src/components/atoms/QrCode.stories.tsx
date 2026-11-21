import type { Meta, StoryObj } from "@storybook/react-vite";
import { QrCode } from "./QrCode";

const meta = {
  title: "Atoms/QrCode",
  component: QrCode,
  args: { value: "http://192.168.1.23:41873/#token=3f9a1c0be27d44a8b6d15e90c7a2f8e14b0d9c63a5e7f1280b4d6c9a1e3f5072", label: "Pairing QR" },
} satisfies Meta<typeof QrCode>;

export default meta;
type Story = StoryObj<typeof meta>;

export const PairingUrl: Story = {};
export const ShortValue: Story = { args: { value: "WFT1:ABC" } };
