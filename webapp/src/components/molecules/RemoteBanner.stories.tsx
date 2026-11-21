import type { Meta, StoryObj } from "@storybook/react-vite";
import { RemoteBanner } from "./RemoteBanner";

const meta = { title: "Molecules/RemoteBanner", component: RemoteBanner } satisfies Meta<typeof RemoteBanner>;
export default meta;

export const Default: StoryObj<typeof meta> = {};
export const PhoneWidth: StoryObj<typeof meta> = { globals: { viewport: { value: "mobile1", isRotated: false } } };
