import type { Meta, StoryObj } from "@storybook/react-vite";
import { DUNGEON_TABLE, NOW, searchingFor } from "../../api/fixtures";
import { SearchProgress } from "./SearchProgress";

const meta = {
  title: "Organisms/SearchProgress",
  component: SearchProgress,
  args: {
    search: searchingFor(20),
    network: { peers: 5, bootstrap: "remembered" },
    dungeons: DUNGEON_TABLE,
    now: NOW,
    onStop: () => {},
  },
} satisfies Meta<typeof SearchProgress>;
export default meta;
type Story = StoryObj<typeof meta>;

export const JustStarted: Story = {};
export const WidenedToStageTwo: Story = { args: { search: searchingFor(135) } };
export const QuietTopicAtLastStage: Story = {
  args: { search: searchingFor(380, { searching: 1, dungeons: ["blackfathom-deeps"] }) },
};
export const BusyTopic: Story = { args: { search: searchingFor(15, { searching: 312, topics: 3, dungeons: ["the-deadmines", "wailing-caverns", "shadowfang-keep"] }) } };
export const ColdStart: Story = {
  args: { search: searchingFor(5, { searching: null }), network: { peers: 0, bootstrap: "none" } },
};
export const FoundThroughTrustedPeer: Story = { args: { network: { peers: 1, bootstrap: "trusted" } } };
export const StageClockRestartedAfterDrop: Story = {
  args: { search: searchingFor(300, { stage: 0, stageClockStartedAt: NOW - 10_000 }) },
};
export const LimitReached: Story = { args: { search: searchingFor(600, { status: "limit_reached", stage: 4 }) } };
export const NotSearching: Story = { args: { search: { ...searchingFor(0), status: "idle" } } };
