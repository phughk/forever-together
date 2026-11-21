import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, within } from "storybook/test";
import { SearchClock } from "./SearchClock";

const meta = {
  title: "Molecules/SearchClock",
  component: SearchClock,
  parameters: {
    docs: {
      description: {
        component:
          "Elapsed time of a running search as a large tabular number (m:ss), the limit muted beside it, and a thick bar. The last minute turns the bar red.",
      },
    },
  },
  args: { elapsedS: 125, limitS: 600 },
} satisfies Meta<typeof SearchClock>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Running: Story = {
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await expect(c.getByText("2:05")).toBeVisible();
    await expect(c.getByText("of 10:00")).toBeVisible();
  },
};
export const JustStarted: Story = { args: { elapsedS: 3 } };
export const LastMinute: Story = { args: { elapsedS: 560 } };
export const AtTheLimit: Story = { args: { elapsedS: 600 } };
export const PhoneWidth: Story = { globals: { viewport: { value: "mobile1", isRotated: false } }, args: { elapsedS: 545 } };
