import type { Meta, StoryObj } from "@storybook/react-vite";
import { SAMPLE_CODE } from "../../api/fixtures";
import { CodePasteForm } from "./CodePasteForm";

const meta = { title: "Molecules/CodePasteForm", component: CodePasteForm, args: { onSubmit: () => {} } } satisfies Meta<typeof CodePasteForm>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Empty: Story = {};
export const Filled: Story = { args: { initialValue: SAMPLE_CODE } };
export const Busy: Story = { args: { initialValue: SAMPLE_CODE, busy: true } };
export const WithError: Story = {
  args: { initialValue: "WFT2:XYZ", error: "This code comes from a different version. Update the addon or the app so both match." },
};
