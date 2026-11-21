import type { Meta, StoryObj } from "@storybook/react-vite";
import { Button } from "../atoms/Button";
import { Switch } from "../atoms/Toggle";
import { Locked } from "./Locked";

const meta = { title: "Molecules/Locked", component: Locked, args: { locked: true, children: null } } satisfies Meta<typeof Locked>;
export default meta;
type Story = StoryObj<typeof meta>;

export const LockedButton: Story = { args: { children: <Button disabled>Clear</Button> } };
export const LockedSwitch: Story = { args: { children: <Switch checked disabled>On</Switch> } };
export const NotLocked: Story = { args: { locked: false, children: <Button>Clear</Button> } };
