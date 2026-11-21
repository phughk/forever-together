import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, waitFor, within } from "storybook/test";
import { LanSettings, type PairingState } from "./LanSettings";

const URL = "http://192.168.1.23:41873/#token=3f9a1c0be27d44a8b6d15e90c7a2f8e14b0d9c63a5e7f1280b4d6c9a1e3f5072";

const meta = {
  title: "Organisms/LanSettings (Phone link)",
  component: LanSettings,
  args: { lanMode: false, onChange: () => {}, onResetToken: () => {}, pairing: { kind: "off" } },
} satisfies Meta<typeof LanSettings>;
export default meta;
type Story = StoryObj<typeof meta>;

const ONE: PairingState = { kind: "ready", url: URL, address: "192.168.1.23", addresses: ["192.168.1.23"] };
const SEVERAL: PairingState = { kind: "ready", url: URL, address: "192.168.1.23", addresses: ["192.168.1.23", "10.0.0.7", "172.20.10.2"] };

export const Off: Story = {};
/** On, QR not shown until "Add a phone" is pressed. */
export const OnWithoutQr: Story = { args: { lanMode: true, pairing: ONE } };
export const OnWithQr: Story = { args: { lanMode: true, pairing: ONE, defaultShowQr: true } };
export const OnWithSeveralAddresses: Story = { args: { lanMode: true, pairing: SEVERAL, defaultShowQr: true } };
export const OnLoading: Story = { args: { lanMode: true, pairing: { kind: "loading" }, defaultShowQr: true } };
export const PairingError: Story = { args: { lanMode: true, pairing: { kind: "error", message: "No private network address found." } } };
export const SignOutFailed: Story = { args: { lanMode: true, pairing: ONE, resetError: "Could not sign out the phones. Try again." } };
/** Phone: the switch and buttons are disabled with a lock icon. */
export const ViewedOnPhone: Story = { args: { lanMode: true, remote: true, pairing: { kind: "forbidden" } } };
export const ViewedOnPhoneOff: Story = { args: { lanMode: false, remote: true, pairing: { kind: "forbidden" } } };
export const EscapeClosesQr: Story = {
  args: { lanMode: true, pairing: ONE, defaultShowQr: true },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByRole("dialog", { name: "Add a phone" })).toBeVisible();
    // The key handler is attached in an effect after the first render, so press again until it has been.
    await waitFor(async () => {
      await userEvent.keyboard("{Escape}");
      await expect(canvas.queryByRole("dialog")).toBeNull();
    });
  },
};
export const AddAPhoneRevealsQr: Story = {
  args: { lanMode: true, pairing: ONE },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(canvas.getByRole("button", { name: "Add a phone" }));
    await expect(canvas.getByRole("img", { name: /QR code/ })).toBeVisible();
    await userEvent.click(canvas.getByRole("button", { name: "Done" }));
    await expect(canvas.getByRole("button", { name: "Add a phone" })).toBeVisible();
  },
};
