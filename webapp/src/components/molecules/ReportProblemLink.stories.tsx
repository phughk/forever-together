import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, within } from "storybook/test";
import { ISSUES_URL } from "../../lib/links";
import { ReportProblemLink } from "./ReportProblemLink";

const meta = { title: "Molecules/ReportProblemLink", component: ReportProblemLink } satisfies Meta<typeof ReportProblemLink>;
export default meta;
type Story = StoryObj<typeof meta>;

/** Opens the issue form in a new tab. */
export const Default: Story = {
  play: async ({ canvasElement }) => {
    const link = within(canvasElement).getByRole("link", { name: "Report a problem" });
    await expect(link).toHaveAttribute("href", ISSUES_URL);
    await expect(link).toHaveAttribute("target", "_blank");
    await expect(link).toHaveAttribute("rel", "noreferrer");
  },
};
export const NarrowPhone: Story = { decorators: [(Story) => <div style={{ width: 320 }}>{Story()}</div>] };
