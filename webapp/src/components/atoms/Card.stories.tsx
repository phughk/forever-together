import type { Meta, StoryObj } from "@storybook/react-vite";
import { Card } from "./Card";
import { Text } from "./Typography";

const meta = { title: "Atoms/Card", component: Card, args: { title: "Section title", children: <Text>Card content.</Text> } } satisfies Meta<typeof Card>;
export default meta;
type Story = StoryObj<typeof meta>;

export const WithTitle: Story = {};
export const WithHint: Story = { args: { hint: "Only people on your Wi-Fi can reach this page." } };
export const WithoutTitle: Story = { args: { title: undefined } };
