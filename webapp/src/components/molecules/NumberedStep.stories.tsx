import type { Meta, StoryObj } from "@storybook/react-vite";
import { SettingsIcon } from "../atoms/SettingsIcon";
import { NumberedStep } from "./NumberedStep";

const meta = {
  title: "Molecules/NumberedStep",
  component: NumberedStep,
  args: { n: 1, icon: <SettingsIcon name="monitor" size={24} />, children: "Open Settings on your PC" },
  decorators: [
    (Story) => (
      <ol className="ft-steps">
        <Story />
      </ol>
    ),
  ],
} satisfies Meta<typeof NumberedStep>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const LongLabel: Story = {
  args: { n: 3, icon: <SettingsIcon name="qr" size={24} />, children: "Scan the QR code with this phone's camera and open the link it shows" },
};
