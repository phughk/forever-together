import type { Meta, StoryObj } from "@storybook/react-vite";
import { Checkbox, Switch } from "./Toggle";

const meta = { title: "Atoms/Toggle", component: Switch } satisfies Meta<typeof Switch>;
export default meta;

export const SwitchOff: StoryObj = { render: () => <Switch>Phone link</Switch> };
export const SwitchOn: StoryObj = { render: () => <Switch defaultChecked>Phone link</Switch> };
export const SwitchDisabled: StoryObj = { render: () => <Switch disabled>Phone link</Switch> };
export const CheckboxStates: StoryObj = {
  render: () => (
    <div style={{ display: "flex", gap: "1rem" }}>
      <Checkbox aria-label="Unchecked" />
      <Checkbox aria-label="Checked" defaultChecked />
      <Checkbox aria-label="Disabled" disabled />
    </div>
  ),
};
