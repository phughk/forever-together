import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, fn, userEvent, within } from "storybook/test";
import { MAC_ADDONS, WIN_ADDONS } from "../../api/fixtures";
import { AddonPathField } from "./AddonPathField";

const meta = { title: "Molecules/AddonPathField", component: AddonPathField, args: { onSubmit: fn() } } satisfies Meta<typeof AddonPathField>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Empty: Story = {};
export const WithCandidate: Story = { args: { candidates: [MAC_ADDONS] } };
export const WithSeveralCandidates: Story = {
  args: { candidates: [MAC_ADDONS, WIN_ADDONS, "/Applications/World of Warcraft/_forever_beta_/Interface/AddOns"] },
};
export const WithCancel: Story = { args: { onCancel: fn() } };
export const Refused: Story = {
  args: {
    initialValue: "/Applications/World of Warcraft/_classic_/Interface",
    error: "/Applications/World of Warcraft/_classic_/Interface is not a World of Warcraft AddOns folder. It should end in Interface/AddOns.",
  },
};
export const Busy: Story = { args: { initialValue: MAC_ADDONS, busy: true, candidates: [MAC_ADDONS] } };
export const LongPath: Story = {
  args: { candidates: ["C:\\Program Files (x86)\\World of Warcraft\\_forever_\\Interface\\AddOns\\.."], initialValue: WIN_ADDONS },
};
export const PasteAndSubmit: Story = {
  play: async ({ canvasElement, args }) => {
    const c = within(canvasElement);
    await expect(c.getByRole("button", { name: "Use folder" })).toBeDisabled();
    await userEvent.type(c.getByLabelText("AddOns folder"), `  ${MAC_ADDONS}  `);
    await userEvent.click(c.getByRole("button", { name: "Use folder" }));
    await expect(args.onSubmit).toHaveBeenCalledWith(MAC_ADDONS);
  },
};
export const CandidateIsOneTap: Story = {
  args: { candidates: [MAC_ADDONS] },
  play: async ({ canvasElement, args }) => {
    await userEvent.click(within(canvasElement).getByRole("button", { name: MAC_ADDONS }));
    await expect(args.onSubmit).toHaveBeenCalledWith(MAC_ADDONS);
  },
};
