import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, waitFor, within } from "storybook/test";
import { Card } from "./Card";
import { INFO_TIP_MAX_CHARS, InfoTip } from "./InfoTip";

const SHORT = "Phones must be on the same Wi-Fi as this computer.";
const AT_CAP = ("Exactly at the cap. " + "x".repeat(INFO_TIP_MAX_CHARS)).slice(0, INFO_TIP_MAX_CHARS);
const OVER_CAP = AT_CAP + " This sentence pushes the tip over the cap and logs a development warning.";

const meta = {
  title: "Atoms/InfoTip",
  component: InfoTip,
  args: { label: "About phone link", children: SHORT },
  parameters: { layout: "padded" },
} satisfies Meta<typeof InfoTip>;
export default meta;
type Story = StoryObj<typeof meta>;

const openIt = (canvasElement: HTMLElement) => userEvent.click(within(canvasElement).getByRole("button", { name: "About phone link" }));

export const Closed: Story = {};

export const Open: Story = {
  play: async ({ canvasElement }) => {
    await openIt(canvasElement);
    await expect(within(canvasElement).getByRole("tooltip")).toBeVisible();
  },
};

export const AtTheCap: Story = {
  args: { children: AT_CAP },
  play: async ({ canvasElement }) => {
    await openIt(canvasElement);
    await expect(within(canvasElement).getByRole("tooltip").textContent).toHaveLength(INFO_TIP_MAX_CHARS);
  },
};

/** Over the cap: shows the text anyway and warns in the console during development. */
export const OverTheCap: Story = {
  args: { children: OVER_CAP },
  play: async ({ canvasElement }) => {
    await openIt(canvasElement);
    await expect(within(canvasElement).getByRole("tooltip")).toBeVisible();
  },
};

export const NextToALabel: Story = {
  render: (args) => (
    <span style={{ display: "inline-flex", alignItems: "center", gap: "0.5rem" }}>
      <span className="ft-label" style={{ margin: 0 }}>Phone link</span>
      <InfoTip {...args} />
    </span>
  ),
};

export const InsideACardTitle: Story = {
  render: () => (
    <Card title="Phone access" hint={SHORT}>
      <span>Card content.</span>
    </Card>
  ),
};

/** Placed at the far right edge so the popover must clamp inside a 390 px viewport. */
export const NarrowPhoneWidth: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  render: (args) => (
    <div style={{ display: "flex", justifyContent: "flex-end", maxWidth: 390 }}>
      <InfoTip {...args}>{AT_CAP}</InfoTip>
    </div>
  ),
  play: async ({ canvasElement }) => {
    await openIt(canvasElement);
    const pop = within(canvasElement).getByRole("tooltip");
    await waitFor(() => {
      const r = pop.getBoundingClientRect();
      expect(r.left).toBeGreaterThanOrEqual(0);
      expect(r.right).toBeLessThanOrEqual(document.documentElement.clientWidth);
    });
  },
};

export const OpensAndClosesWithClick: Story = {
  play: async ({ canvasElement }) => {
    const button = within(canvasElement).getByRole("button", { name: "About phone link" });
    await userEvent.click(button);
    await expect(button).toHaveAttribute("aria-expanded", "true");
    await expect(within(canvasElement).getByRole("tooltip")).toBeVisible();
    await userEvent.click(button);
    await expect(button).toHaveAttribute("aria-expanded", "false");
    await expect(within(canvasElement).queryByRole("tooltip")).toBeNull();
  },
};

export const ClosesOnEscape: Story = {
  play: async ({ canvasElement }) => {
    await openIt(canvasElement);
    await expect(within(canvasElement).getByRole("tooltip")).toBeVisible();
    await userEvent.keyboard("{Escape}");
    await expect(within(canvasElement).queryByRole("tooltip")).toBeNull();
  },
};

export const ClosesOnOutsideClick: Story = {
  render: (args) => (
    <div>
      <InfoTip {...args} />
      <p data-testid="outside">Somewhere else on the page.</p>
    </div>
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await openIt(canvasElement);
    await expect(canvas.getByRole("tooltip")).toBeVisible();
    await userEvent.click(canvas.getByTestId("outside"));
    await expect(canvas.queryByRole("tooltip")).toBeNull();
  },
};

export const OpensWithKeyboard: Story = {
  play: async ({ canvasElement }) => {
    await userEvent.tab();
    await userEvent.keyboard("{Enter}");
    await expect(within(canvasElement).getByRole("tooltip")).toBeVisible();
  },
};
