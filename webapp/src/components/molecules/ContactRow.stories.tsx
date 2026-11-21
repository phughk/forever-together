import type { Meta, StoryObj } from "@storybook/react-vite";
import { ContactRow } from "./ContactRow";

const KEY = "a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90";
const meta = { title: "Molecules/ContactRow", component: ContactRow, args: { contactKey: KEY, onRemove: () => {} } } satisfies Meta<typeof ContactRow>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
/** Phone: the list is readable and the key can be copied, but not removed. */
export const ReadOnly: Story = { args: { onRemove: undefined } };
