import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect } from "storybook/test";
import { INFO_TIP_MAX_CHARS } from "../atoms/InfoTip";
import { SEARCHING_WIDER_TIP, SearchingWiderChip } from "./SearchingWiderChip";

const meta = {
  title: "Molecules/SearchingWiderChip",
  component: SearchingWiderChip,
  parameters: {
    docs: { description: { component: "Shown once the search has widened. The rule is in the info tip. There is no countdown to the next step." } },
  },
} satisfies Meta<typeof SearchingWiderChip>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  play: async () => {
    await expect(SEARCHING_WIDER_TIP.length).toBeLessThanOrEqual(INFO_TIP_MAX_CHARS);
  },
};
