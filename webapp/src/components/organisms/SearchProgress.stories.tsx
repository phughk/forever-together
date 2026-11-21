import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, fn, userEvent, within } from "storybook/test";
import { DUNGEON_TABLE, NOW, makeAttempts, makeDiagnosis, searchingFor } from "../../api/fixtures";
import { SearchProgress } from "./SearchProgress";

const meta = {
  title: "Organisms/SearchProgress",
  component: SearchProgress,
  parameters: {
    docs: {
      description: {
        component: [
          "Running: Stop at the top, one bar for time to the 10 minute limit, a \"Searching wider\" chip once the search has widened (rule in an info tip, no countdown), and what is seen from here (see DiagnosisSummary).",
          "The connection is not shown here; the header's status dot covers it, so `network` is accepted but unused.",
          "Ended: title \"Search ended\", the result, and \"Search again\" and \"Change dungeon\". Without those two callbacks the card has a single \"OK\".",
          "An older server sends no diagnosis: the card shows only a rough count of players.",
        ].join("\n"),
      },
    },
  },
  args: {
    search: searchingFor(20),
    dungeons: DUNGEON_TABLE,
    now: NOW,
    onStop: fn(),
  },
} satisfies Meta<typeof SearchProgress>;
export default meta;
type Story = StoryObj<typeof meta>;

// --- Running ---

export const JustStarted: Story = {};
/** Stage above 0: the "Searching wider" chip shows. */
export const SearchingWider: Story = { args: { search: searchingFor(135) } };
export const NoWiderChipAtStageZero: Story = { args: { search: searchingFor(50) } };
export const QuietTopicAtLastStage: Story = {
  args: { search: searchingFor(380, { searching: 1, dungeons: ["blackfathom-deeps"] }) },
};
export const NearTheLimit: Story = { args: { search: searchingFor(590) } };

const running = (situation: string, over = {}, elapsed = 120) => ({
  search: searchingFor(elapsed, { searching: 8, diagnosis: makeDiagnosis(situation, over) }),
});
export const WarmingUp: Story = { args: { search: searchingFor(4, { searching: null }) } };
export const NotConnected: Story = { args: { search: searchingFor(40, { searching: 0, diagnosis: makeDiagnosis("not_connected") }) } };
export const Alone: Story = { args: running("alone", {}, 200) };
export const NoTank: Story = { args: running("no_tank") };
export const NoHealer: Story = { args: running("no_healer") };
export const ShortOnDamage: Story = { args: running("short_on_damage") };
export const HealthyLooking: Story = { args: running("healthy") };
export const FullSample: Story = { args: running("healthy", { sampleFull: true, seen: { players: 64, tanks: 8, healers: 9, damage: 47 } }) };
export const PartialSample: Story = { args: running("no_tank", { sampleFull: false, seen: { players: 3, tanks: 0, healers: 1, damage: 2 } }) };
export const UnknownSituation: Story = { args: running("needs_a_new_code") };
export const OthersGrouping: Story = { args: running("others_grouping") };
export const OthersGroupingNoAttempts: Story = { args: running("others_grouping", { attempts: undefined }) };
export const NobodyAnswered: Story = { args: running("nobody_answered") };
export const NobodyAnsweredNoAttempts: Story = { args: running("nobody_answered", { attempts: undefined }) };
export const HealthyWithAttempts: Story = { args: running("healthy", { attempts: makeAttempts({ invitesSent: 5, acceptsGot: 2 }) }) };
export const HealthyWithoutAttempts: Story = { args: running("healthy", { attempts: undefined }) };
export const OutlookRarelyFoundNoTank: Story = { args: running("no_tank", { outlook: "rarely_found" }) };
export const OutlookUsuallyFoundHealthy: Story = { args: running("healthy", { outlook: "usually_found" }) };
export const MultiDungeon: Story = {
  args: {
    search: searchingFor(150, {
      searching: 312,
      topics: 3,
      dungeons: ["the-deadmines", "wailing-caverns", "shadowfang-keep"],
      diagnosis: makeDiagnosis("no_healer"),
    }),
  },
};
export const BusyTopic: Story = {
  args: { search: searchingFor(15, { searching: 312, topics: 3, dungeons: ["the-deadmines", "wailing-caverns", "shadowfang-keep"] }) },
};
export const StageClockRestartedAfterDrop: Story = {
  args: { search: searchingFor(300, { stage: 0, stageClockStartedAt: NOW - 10_000 }) },
};
/** The `network` prop is still accepted, but nothing about the network shows in the card. */
export const NetworkPropIgnored: Story = { args: { network: { peers: 0, bootstrap: "none" } } };

/** An older server sends no diagnosis: only a rough count of players. */
export const NoDiagnosisFromOldServer: Story = {
  args: { search: searchingFor(380, { searching: 1, diagnosis: null, dungeons: ["blackfathom-deeps"] }) },
};
export const NoDiagnosisCounting: Story = { args: { search: searchingFor(5, { searching: null, diagnosis: null }) } };
export const NoDiagnosisBusy: Story = { args: { search: searchingFor(30, { diagnosis: null }) } };

/** Stop is the first control in the card, so it stays above the fold on a laptop. */
export const StopIsPinnedAtTheTop: Story = {
  args: running("no_tank", { outlook: "rarely_found" }, 300),
  play: async ({ canvasElement, args }) => {
    const c = within(canvasElement);
    const stop = c.getByRole("button", { name: "Stop search" });
    await expect(stop.getBoundingClientRect().top - c.getByRole("heading", { name: "Searching" }).getBoundingClientRect().top).toBeLessThan(60);
    await userEvent.click(stop);
    await expect(args.onStop).toHaveBeenCalled();
  },
};
export const PhoneWidth: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: {
    search: searchingFor(300, {
      topics: 3,
      dungeons: ["the-deadmines", "wailing-caverns", "shadowfang-keep"],
      diagnosis: makeDiagnosis("nobody_answered", { sampleFull: true, outlook: "rarely_found" }),
    }),
  },
};

// --- Ended ---

const ended = (situation: string, over = {}) => ({
  search: searchingFor(600, { status: "limit_reached", stage: 4, diagnosis: makeDiagnosis(situation, { ended: true, ...over }) }),
});
const bothActions = { onSearchAgain: fn(), onChangeDungeon: fn() };

export const EndedNotConnected: Story = { args: ended("not_connected") };
export const EndedAlone: Story = { args: ended("alone") };
export const EndedNoTank: Story = { args: { ...ended("no_tank"), ...bothActions } };
export const EndedNoHealer: Story = { args: ended("no_healer") };
export const EndedShortOnDamage: Story = { args: ended("short_on_damage") };
export const EndedHealthyLooking: Story = { args: ended("healthy") };
export const EndedUnknownSituation: Story = { args: ended("needs_a_new_code") };
export const EndedOthersGrouping: Story = { args: ended("others_grouping", { attempts: makeAttempts({ takenSeen: 25 }) }) };
export const EndedOthersGroupingNoAttempts: Story = { args: ended("others_grouping", { attempts: undefined }) };
export const EndedNobodyAnswered: Story = { args: ended("nobody_answered", { attempts: makeAttempts({ invitesSent: 18 }) }) };
export const EndedNobodyAnsweredNoAttempts: Story = { args: ended("nobody_answered", { attempts: undefined }) };
export const EndedMultiDungeonFullSample: Story = {
  args: {
    search: searchingFor(600, {
      status: "limit_reached",
      stage: 4,
      topics: 3,
      dungeons: ["the-deadmines", "wailing-caverns", "shadowfang-keep"],
      diagnosis: makeDiagnosis("no_tank", { ended: true, sampleFull: true, seen: { players: 64, tanks: 0, healers: 12, damage: 52 } }),
    }),
    ...bothActions,
  },
};
export const EndedNoDiagnosisFromOldServer: Story = {
  args: { search: searchingFor(600, { status: "limit_reached", stage: 4, diagnosis: null }) },
};
export const EndedPhoneWidth: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { ...ended("nobody_answered", { attempts: makeAttempts({ invitesSent: 18 }) }), ...bothActions },
};

/** With both callbacks the card offers "Search again" and "Change dungeon", and no "OK". */
export const EndedWithNextSteps: Story = {
  args: { ...ended("no_tank"), ...bothActions },
  play: async ({ canvasElement, args }) => {
    const c = within(canvasElement);
    await expect(c.getByRole("heading", { name: "Search ended" })).toBeVisible();
    await expect(c.queryByRole("button", { name: "OK" })).toBeNull();
    await userEvent.click(c.getByRole("button", { name: "Search again" }));
    await expect(args.onSearchAgain).toHaveBeenCalled();
    await userEvent.click(c.getByRole("button", { name: "Change dungeon" }));
    await expect(args.onChangeDungeon).toHaveBeenCalled();
  },
};
/** Without the callbacks the card falls back to a single "OK" that calls `onStop`. */
export const EndedFallbackOk: Story = {
  args: ended("no_tank"),
  play: async ({ canvasElement, args }) => {
    const c = within(canvasElement);
    await userEvent.click(c.getByRole("button", { name: "OK" }));
    await expect(args.onStop).toHaveBeenCalled();
  },
};

export const NotSearching: Story = { args: { search: { ...searchingFor(0), status: "idle" } } };
