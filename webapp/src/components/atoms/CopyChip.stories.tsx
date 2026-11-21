import type { Meta, StoryObj } from "@storybook/react-vite";
import { CopyChip } from "./CopyChip";

const meta = { title: "Atoms/CopyChip", component: CopyChip, args: { text: "/wowft" } } satisfies Meta<typeof CopyChip>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const LongText: Story = { args: { text: "/wowft refresh now please" } };
