import type { Meta, StoryObj } from "@storybook/react-vite";
import { Text } from "../atoms/Typography";
import { Details } from "./Details";

const meta = {
  title: "Molecules/Details",
  component: Details,
  args: { summary: "Not working? Change address", children: <Text muted>Pick the network your phone is on.</Text> },
} satisfies Meta<typeof Details>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Closed: Story = {};
export const Open: Story = { args: { defaultOpen: true } };
export const LongContent: Story = {
  args: {
    defaultOpen: true,
    summary: "A summary line that is quite long and has to wrap on a narrow phone screen without breaking the layout",
    children: (
      <>
        {Array.from({ length: 6 }, (_, i) => (
          <Text key={i} muted>
            Paragraph {i + 1}. Long content wraps inside the section and never pushes the page sideways, even with an_unbroken_token_that_is_very_long_indeed_and_keeps_going.
          </Text>
        ))}
      </>
    ),
  },
};
