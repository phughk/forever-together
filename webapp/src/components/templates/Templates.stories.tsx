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
      header={<Slot label="header: gear (40px)" height={40} />}
      notices={<Slot label="notices (1 to 2 lines)" height={40} />}
      strip={<Slot label="strip: one-line character summary (shares the top row with the header)" height={40} />}
      main={<Slot label="main: full width, the dungeon card grid scrolls inside" height={420} />}
    />
  ),
};

export const FinderWithoutStripOrNotices: StoryObj = {
  render: () => <FinderTemplate header={<Slot label="header" height={40} />} main={<Slot label="main" height={420} />} />,
};

/** Searching, party and ended: one card at a readable width. From 1100px the strip becomes a side column. */
export const FinderFocusLayout: StoryObj = {
  render: () => (
    <FinderTemplate
      layout="focus"
      header={<Slot label="header: gear" height={40} />}
      strip={<Slot label="strip: character (side column when wide)" height={64} />}
      main={<Slot label="main: one card (up to 46rem)" height={420} />}
    />
  ),
};

export const FinderFocusAtPhoneWidth: StoryObj = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  render: FinderFocusLayout.render,
};

export const FinderAtPhoneWidth: StoryObj = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  render: FinderWithAllSlots.render,
};

export const SettingsAtPhoneWidth: StoryObj = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  render: () => (
    <SettingsTemplate
      sections={
        <>
          <Slot label="section: phone link" height={140} />
          <Slot label="section: addon" height={120} />
        </>
      }
    />
  ),
};

export const SettingsNarrow: StoryObj = {
  render: () => <SettingsTemplate narrow sections={<Slot label="one card (not-linked screen), 32rem" height={200} />} />,
};

export const Settings: StoryObj = {
  render: () => (
    <SettingsTemplate
      header={<Slot label="header: Back arrow" height={40} />}
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
