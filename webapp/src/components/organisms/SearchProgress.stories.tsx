import type { Meta, StoryObj } from "@storybook/react-vite";
import { DUNGEON_TABLE, NOW, makeAttempts, makeDiagnosis, searchingFor } from "../../api/fixtures";
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

const running = (situation: string, over = {}, elapsed = 120) =>
  ({ search: searchingFor(elapsed, { searching: 8, diagnosis: makeDiagnosis(situation, over) }) });
export const OutlookRarelyFoundNoTank: Story = { args: running("no_tank", { outlook: "rarely_found" }) };
export const OutlookUsuallyFoundHealthy: Story = { args: running("healthy", { outlook: "usually_found" }) };
export const NoOtherComputersConnected: Story = { args: { network: { peers: 0, bootstrap: "none" } } };
export const OneOtherComputerConnected: Story = { args: { network: { peers: 1, bootstrap: "remembered" } } };
export const WarmingUp: Story = { args: { search: searchingFor(4, { searching: null }) } };
export const NotConnected: Story = {
  args: { search: searchingFor(40, { searching: 0, diagnosis: makeDiagnosis("not_connected") }), network: { peers: 0, bootstrap: "none" } },
};
export const Alone: Story = { args: running("alone", {}, 200) };
export const NoTank: Story = { args: running("no_tank") };
export const NoHealer: Story = { args: running("no_healer") };
export const ShortOnDamage: Story = { args: running("short_on_damage") };
export const HealthyLooking: Story = { args: running("healthy") };
export const FullSample: Story = { args: running("healthy", { sampleFull: true, seen: { players: 64, tanks: 8, healers: 9, damage: 47 } }) };
export const UnknownSituation: Story = { args: running("needs_a_new_code") };
/** An older server sends no diagnosis: the card renders as before, with the old low-count notice. */
export const NoDiagnosisFromOldServer: Story = {
  args: { search: searchingFor(380, { searching: 1, diagnosis: null, dungeons: ["blackfathom-deeps"] }) },
};
export const NoDiagnosisBusy: Story = { args: { search: searchingFor(30, { diagnosis: null }) } };
export const BusyTopic: Story = { args: { search: searchingFor(15, { searching: 312, topics: 3, dungeons: ["the-deadmines", "wailing-caverns", "shadowfang-keep"] }) } };
export const ColdStart: Story = {
  args: { search: searchingFor(5, { searching: null }), network: { peers: 0, bootstrap: "none" } },
};
export const FoundThroughTrustedComputer: Story = { args: { network: { peers: 1, bootstrap: "trusted" } } };
export const StageClockRestartedAfterDrop: Story = {
  args: { search: searchingFor(300, { stage: 0, stageClockStartedAt: NOW - 10_000 }) },
};
const ended = (situation: string, over = {}) =>
  ({ search: searchingFor(600, { status: "limit_reached", stage: 4, diagnosis: makeDiagnosis(situation, { ended: true, ...over }) }) });
export const EndedNotConnected: Story = { args: { ...ended("not_connected"), network: { peers: 0, bootstrap: "none" } } };
export const EndedAlone: Story = { args: ended("alone") };
export const EndedNoTank: Story = { args: ended("no_tank") };
export const EndedNoHealer: Story = { args: ended("no_healer") };
export const EndedShortOnDamage: Story = { args: ended("short_on_damage") };
export const EndedHealthyLooking: Story = { args: ended("healthy") };
export const EndedMultiDungeonFullSample: Story = {
  args: {
    search: searchingFor(600, {
      status: "limit_reached",
      stage: 4,
      topics: 3,
      dungeons: ["the-deadmines", "wailing-caverns", "shadowfang-keep"],
      diagnosis: makeDiagnosis("no_tank", { ended: true, sampleFull: true, seen: { players: 64, tanks: 0, healers: 12, damage: 52 } }),
    }),
  },
};
export const EndedNoDiagnosisFromOldServer: Story = {
  args: { search: searchingFor(600, { status: "limit_reached", stage: 4, diagnosis: null }) },
};
export const NotSearching: Story = { args: { search: { ...searchingFor(0), status: "idle" } } };

export const OthersGrouping: Story = { args: running("others_grouping") };
export const OthersGroupingNoAttempts: Story = { args: running("others_grouping", { attempts: undefined }) };
export const NobodyAnswered: Story = { args: running("nobody_answered") };
export const NobodyAnsweredNoAttempts: Story = { args: running("nobody_answered", { attempts: undefined }) };
export const HealthyWithAttempts: Story = { args: running("healthy", { attempts: makeAttempts({ invitesSent: 5, acceptsGot: 2 }) }) };
export const EndedOthersGrouping: Story = { args: ended("others_grouping", { attempts: makeAttempts({ takenSeen: 25 }) }) };
export const EndedOthersGroupingNoAttempts: Story = { args: ended("others_grouping", { attempts: undefined }) };
export const EndedNobodyAnswered: Story = { args: ended("nobody_answered", { attempts: makeAttempts({ invitesSent: 18 }) }) };
export const EndedNobodyAnsweredNoAttempts: Story = { args: ended("nobody_answered", { attempts: undefined }) };
export const EndedNobodyAnsweredMultiDungeon: Story = {
  args: {
    search: searchingFor(600, {
      status: "limit_reached",
      stage: 4,
      topics: 3,
      dungeons: ["the-deadmines", "wailing-caverns", "shadowfang-keep"],
      diagnosis: makeDiagnosis("nobody_answered", { ended: true, attempts: makeAttempts({ invitesSent: 18 }) }),
    }),
  },
};
