import type { Meta, StoryObj } from "@storybook/react-vite";
import { Entity, Heading, Label, Text } from "./Typography";

const meta = { title: "Atoms/Typography", component: Heading } satisfies Meta<typeof Heading>;
export default meta;

export const Headings: StoryObj = {
  render: () => (
    <div style={{ display: "grid", gap: "0.75rem" }}>
      <Heading level={1}>Level 1 heading</Heading>
      <Heading level={2}>Level 2 heading</Heading>
      <Heading level={3}>Level 3 heading</Heading>
    </div>
  ),
};

export const BodyText: StoryObj = {
  render: () => (
    <div style={{ display: "grid", gap: "0.5rem", maxWidth: "32rem" }}>
      <Text>Body text for explanations and descriptions.</Text>
      <Text muted>Muted text for secondary information.</Text>
      <Text size="small">Small text for captions.</Text>
      <Text mono size="small">WFT1:KJ2 8E4%KDJ5EDGX</Text>
    </div>
  ),
};

export const FormLabel: StoryObj = {
  render: () => <Label htmlFor="x">Form label</Label>,
};

export const LongUnbrokenText: StoryObj = {
  render: () => (
    <div style={{ maxWidth: "16rem" }}>
      <Heading level={2}>Averyveryveryveryveryveryverylongheadingwithoutspaces</Heading>
    </div>
  ),
};

const pairStyle = { display: "grid", gridTemplateColumns: "auto 1fr 1fr", gap: "0.5rem 1.5rem", alignItems: "baseline" } as const;

export const EntityVersusUi: StoryObj = {
  render: () => (
    <div style={pairStyle}>
      <span />
      <Text muted size="small">Entity font (Lora)</Text>
      <Text muted size="small">UI font (Overpass)</Text>
      <Text muted size="small">Character</Text>
      <Text><Entity>Testa Fakename</Entity></Text>
      <Text>Testa Fakename</Text>
      <Text muted size="small">Dungeon</Text>
      <Text><Entity>Testville Depths</Entity></Text>
      <Text>Testville Depths</Text>
      <Text muted size="small">Sentence</Text>
      <Text>
        <Entity>Testa Fakename</Entity> joined 3/5 for <Entity>Testville Depths</Entity>, level 24.
      </Text>
      <Text>Testa Fakename joined 3/5 for Testville Depths, level 24.</Text>
    </div>
  ),
};

export const EntityLongName: StoryObj = {
  render: () => (
    <div style={{ maxWidth: "16rem" }}>
      <Text>
        <Entity>Testa Fakenameofthehighlandsandthelongwinding</Entity>
      </Text>
      <Text>
        <Entity as="strong">The Extraordinarily Long Testville Depths of Lower Fakeland</Entity>
      </Text>
    </div>
  ),
};

export const EntityPhoneWidth: StoryObj = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
  render: () => (
    <div style={{ width: "20rem", maxWidth: "100%" }}>
      <Text>
        <Entity>Testa Fakename</Entity> is waiting in <Entity>Testville Depths</Entity> as a healer.
      </Text>
    </div>
  ),
};
