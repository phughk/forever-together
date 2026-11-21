import type { Meta, StoryObj } from "@storybook/react-vite";
import { Heading, Label, Text } from "./Typography";

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
