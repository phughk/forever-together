import type { Meta, StoryObj } from "@storybook/react-vite";
import { FinderTemplate } from "./FinderTemplate";
import { SettingsTemplate } from "./SettingsTemplate";

const Slot = ({ label, height }: { label: string; height: number }) => (
  <div
    style={{
      minHeight: height,
      display: "grid",
      placeItems: "center",
      border: "2px dashed var(--border)",
      borderRadius: 8,
      color: "var(--text-muted)",
      font: "0.85rem var(--font)",
      background: "var(--surface)",
    }}
  >
    {label}
  </div>
);

const meta = { title: "Templates/Layouts", component: FinderTemplate, parameters: { layout: "fullscreen" } } satisfies Meta<typeof FinderTemplate>;
export default meta;

export const FinderWithAllSlots: StoryObj = {
  render: () => (
    <FinderTemplate
      header={<Slot label="header (one row, about 3.5rem)" height={56} />}
      notices={<Slot label="notices (1 to 2 lines)" height={40} />}
      strip={<Slot label="strip: one-line character summary (at most about 5rem)" height={64} />}
      main={<Slot label="main: one card for the current screen (dungeon list scrolls inside)" height={420} />}
    />
  ),
};

export const FinderWithoutStripOrNotices: StoryObj = {
  render: () => <FinderTemplate header={<Slot label="header" height={56} />} main={<Slot label="main" height={420} />} />,
};

export const FinderAtPhoneWidth: StoryObj = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  render: FinderWithAllSlots.render,
};

export const Settings: StoryObj = {
  render: () => (
    <SettingsTemplate
      header={<Slot label="header" height={56} />}
      notices={<Slot label="notices (remote banner, error)" height={40} />}
      sections={
        <>
          <Slot label="section: phone link (main action, about 8 to 14rem)" height={140} />
          <Slot label="section: addon (badge, values line, one button, about 8rem)" height={120} />
          <Slot label="section: advanced (collapsed, one row)" height={56} />
        </>
      }
    />
  ),
};
