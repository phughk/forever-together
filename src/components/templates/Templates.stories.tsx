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

const meta = { title: "Templates/Layouts", component: FinderTemplate } satisfies Meta<typeof FinderTemplate>;
export default meta;

export const FinderWithAllSlots: StoryObj = {
  render: () => (
    <FinderTemplate
      header={<Slot label="header (one row)" height={56} />}
      notices={<Slot label="notices (1 to 2 lines)" height={40} />}
      top={<Slot label="top: search progress or party (full width)" height={160} />}
      left={<Slot label="left column (about 22rem): character" height={320} />}
      right={<Slot label="right column: dungeons (1 to 30 rows)" height={420} />}
    />
  ),
};

export const FinderWithoutTopOrNotices: StoryObj = {
  render: () => (
    <FinderTemplate
      header={<Slot label="header" height={56} />}
      left={<Slot label="left column" height={320} />}
      right={<Slot label="right column" height={420} />}
    />
  ),
};

export const FinderAtPhoneWidth: StoryObj = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  render: FinderWithAllSlots.render,
};

export const Settings: StoryObj = {
  render: () => (
    <SettingsTemplate
      header={<Slot label="header" height={56} />}
      notices={<Slot label="notices" height={40} />}
      sections={
        <>
          <Slot label="section: LAN mode" height={140} />
          <Slot label="section: paired phones" height={120} />
          <Slot label="section: peers" height={220} />
        </>
      }
    />
  ),
};
