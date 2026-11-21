import type { Meta, StoryObj } from "@storybook/react-vite";
import { Notice } from "./Notice";

const meta = { title: "Atoms/Notice", component: Notice, args: { children: "Something worth knowing." } } satisfies Meta<typeof Notice>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Info: Story = {};
export const Warn: Story = { args: { tone: "warn", children: "Your code is 40 minutes old." } };
export const Bad: Story = { args: { tone: "bad", children: "The code has a typo." } };
export const Good: Story = { args: { tone: "good", children: "Everyone confirmed." } };
