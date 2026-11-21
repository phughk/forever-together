import type { Meta, StoryObj } from "@storybook/react-vite";
import { NotPairedPage } from "./NotPairedPage";

const meta = { title: "Pages/NotPaired", component: NotPairedPage, parameters: { layout: "fullscreen" } } satisfies Meta<typeof NotPairedPage>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Three numbered steps, no header and no navigation. */
export const Steps: Story = {};
export const OnPhone: Story = { globals: { viewport: { value: "mobile1", isRotated: false } } };
