import type { Meta, StoryObj } from "@storybook/react-vite";
import { PairingQr } from "./PairingQr";

const meta = {
  title: "Molecules/PairingQr",
  component: PairingQr,
  args: { url: "http://192.168.1.23:41873/#token=3f9a1c0be27d44a8b6d15e90c7a2f8e14b0d9c63a5e7f1280b4d6c9a1e3f5072" },
} satisfies Meta<typeof PairingQr>;
export default meta;

export const Default: StoryObj<typeof meta> = {};
