import type { Meta, StoryObj } from "@storybook/react-vite";
import { Badge, type Tone } from "./Badge";

const meta = { title: "Atoms/Badge", component: Badge } satisfies Meta<typeof Badge>;
export default meta;

const tones: Tone[] = ["neutral", "good", "warn", "bad", "info", "alliance", "horde"];

export const AllTones: StoryObj = {
  render: () => (
    <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
      {tones.map((t) => (
        <Badge key={t} tone={t}>
          {t}
        </Badge>
      ))}
    </div>
  ),
};
