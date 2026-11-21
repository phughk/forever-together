import type { Meta, StoryObj } from "@storybook/react-vite";
import { makeAttempts, makeDiagnosis } from "../../api/fixtures";
import { DiagnosisSummary } from "./DiagnosisSummary";

const meta = {
  title: "Molecules/DiagnosisSummary",
  component: DiagnosisSummary,
  parameters: {
    docs: {
      description: {
        component: [
          "Tells the player in plain language what this computer can see while a search runs, and in a summary when it ended.",
          "The backend sends a situation code and counts; the sentences are written in `lib/diagnosisCopy.ts`.",
          "",
          "Content limits: the headline is one sentence of at most 220 characters, the detail one sentence of at most 140.",
          "Several dungeons add only the count (\"across 3 dungeons\"), never the names, so a long dungeon list cannot lengthen the sentence.",
          "Wording rules: claims are about what this computer sees, numbers are rough, nothing promises a group, and nothing reassures when a role is missing or the app is not connected.",
          "Attempt counts (\"invited 6, accepted 0\") join the counts row only when the server sends them and this computer invited someone.",
          "An unknown situation code gets a neutral sentence.",
          "",
          "Outlook: while a search runs, a quiet muted line after the detail says how this computer's own earlier searches went (usually, sometimes or rarely found).",
          "It is history shown as a hedged clause, never as data: no numbers, percentages, counts or dates. It stays within 110 characters and never promises a group.",
          "`usually_found` next to a live problem (no tank, no healer, short on damage, nobody answered, alone, not connected, warming up) is dropped, and so is an unknown value. It is never shown for an ended search.",
        ].join("\n"),
      },
    },
  },
  args: { diagnosis: makeDiagnosis("healthy"), elapsedS: 45, dungeonCount: 1 },
} satisfies Meta<typeof DiagnosisSummary>;
export default meta;
type Story = StoryObj<typeof meta>;

export const WarmingUp: Story = { args: { diagnosis: makeDiagnosis("warming_up"), elapsedS: 3 } };
export const NotConnected: Story = { args: { diagnosis: makeDiagnosis("not_connected"), elapsedS: 40 } };
export const Alone: Story = { args: { diagnosis: makeDiagnosis("alone"), elapsedS: 90 } };
export const NoTank: Story = { args: { diagnosis: makeDiagnosis("no_tank"), elapsedS: 120 } };
export const NoHealer: Story = { args: { diagnosis: makeDiagnosis("no_healer"), elapsedS: 120 } };
export const ShortOnDamage: Story = { args: { diagnosis: makeDiagnosis("short_on_damage"), elapsedS: 120 } };
export const HealthyLooking: Story = {};

export const NoTankAndYouAreTheTank: Story = {
  name: "No tank (own role is tank)",
  args: { diagnosis: makeDiagnosis("no_tank", { ownRole: "tank" }) },
};

export const EndedNotConnected: Story = { args: { diagnosis: makeDiagnosis("not_connected", { ended: true }), elapsedS: 600 } };
export const EndedAlone: Story = { args: { diagnosis: makeDiagnosis("alone", { ended: true }), elapsedS: 600 } };
export const EndedNoTank: Story = { args: { diagnosis: makeDiagnosis("no_tank", { ended: true }), elapsedS: 600 } };
export const EndedNoHealer: Story = { args: { diagnosis: makeDiagnosis("no_healer", { ended: true }), elapsedS: 600 } };
export const EndedShortOnDamage: Story = { args: { diagnosis: makeDiagnosis("short_on_damage", { ended: true }), elapsedS: 600 } };
export const EndedHealthyLooking: Story = { args: { diagnosis: makeDiagnosis("healthy", { ended: true }), elapsedS: 600 } };
export const EndedWarmingUp: Story = { args: { diagnosis: makeDiagnosis("warming_up", { ended: true }), elapsedS: 600 } };

export const FullSample: Story = {
  name: "Sample full (rough counts)",
  args: { diagnosis: makeDiagnosis("healthy", { sampleFull: true, seen: { players: 64, tanks: 8, healers: 9, damage: 47 } }) },
};
export const FullSampleEnded: Story = {
  args: { diagnosis: makeDiagnosis("no_tank", { ended: true, sampleFull: true, seen: { players: 64, tanks: 0, healers: 12, damage: 52 } }), elapsedS: 600 },
};
export const OneOtherPlayer: Story = {
  args: { diagnosis: makeDiagnosis("alone", { seen: { players: 1, tanks: 0, healers: 0, damage: 1 } }) },
};

export const MultiDungeon: Story = { args: { dungeonCount: 3, diagnosis: makeDiagnosis("no_healer") } };
export const MultiDungeonEnded: Story = {
  args: { dungeonCount: 8, diagnosis: makeDiagnosis("no_healer", { ended: true, sampleFull: true }), elapsedS: 600 },
};

/** One sentence only: no detail line. */
export const OneSentence: Story = { args: { diagnosis: makeDiagnosis("warming_up") } };
/** Headline plus detail: the longest copy the molecule shows. */
export const TwoSentencesLongest: Story = {
  args: { dungeonCount: 12, diagnosis: makeDiagnosis("no_healer", { ended: true, sampleFull: true, ownRole: "healer", seen: { players: 128, tanks: 31, healers: 0, damage: 97 } }), elapsedS: 600 },
};

export const UnknownSituationRunning: Story = { args: { diagnosis: makeDiagnosis("needs_a_new_code") } };
export const UnknownSituationEnded: Story = { args: { diagnosis: makeDiagnosis("needs_a_new_code", { ended: true }), elapsedS: 600 } };

export const PhoneWidth: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { dungeonCount: 3, diagnosis: makeDiagnosis("no_tank", { ended: true }), elapsedS: 600 },
};

export const OthersGrouping: Story = { args: { diagnosis: makeDiagnosis("others_grouping"), elapsedS: 90 } };
export const OthersGroupingNoAttempts: Story = {
  name: "Others grouping (no attempts from an older server)",
  args: { diagnosis: makeDiagnosis("others_grouping", { attempts: undefined }), elapsedS: 90 },
};
export const EndedOthersGrouping: Story = {
  args: { diagnosis: makeDiagnosis("others_grouping", { ended: true, attempts: makeAttempts({ takenSeen: 25 }) }), elapsedS: 600 },
};
export const EndedOthersGroupingNoAttempts: Story = {
  args: { diagnosis: makeDiagnosis("others_grouping", { ended: true, attempts: undefined }), elapsedS: 600 },
};
export const NobodyAnswered: Story = { args: { diagnosis: makeDiagnosis("nobody_answered"), elapsedS: 120 } };
export const NobodyAnsweredNoAttempts: Story = {
  name: "Nobody answered (no attempts from an older server)",
  args: { diagnosis: makeDiagnosis("nobody_answered", { attempts: undefined }), elapsedS: 120 },
};
export const EndedNobodyAnswered: Story = {
  args: { diagnosis: makeDiagnosis("nobody_answered", { ended: true, attempts: makeAttempts({ invitesSent: 18 }) }), elapsedS: 600 },
};
export const EndedNobodyAnsweredNoAttempts: Story = {
  args: { diagnosis: makeDiagnosis("nobody_answered", { ended: true, attempts: undefined }), elapsedS: 600 },
};
export const NobodyAnsweredMultiDungeon: Story = {
  args: { dungeonCount: 3, diagnosis: makeDiagnosis("nobody_answered", { attempts: makeAttempts({ invitesSent: 12 }) }) },
};
export const OthersGroupingMultiDungeonEnded: Story = {
  args: { dungeonCount: 3, diagnosis: makeDiagnosis("others_grouping", { ended: true, attempts: makeAttempts({ takenSeen: 40 }) }), elapsedS: 600 },
};
export const HealthyWithAttempts: Story = {
  args: { diagnosis: makeDiagnosis("healthy", { attempts: makeAttempts({ invitesSent: 5, acceptsGot: 2 }) }) },
};
export const EndedHealthyWithAttempts: Story = {
  args: { diagnosis: makeDiagnosis("healthy", { ended: true, attempts: makeAttempts({ invitesSent: 9, acceptsGot: 3 }) }), elapsedS: 600 },
};
export const AttemptsAllZeroHidden: Story = {
  name: "Attempts all zero (row hidden)",
  args: { diagnosis: makeDiagnosis("healthy", { attempts: makeAttempts() }) },
};
export const NobodyAnsweredPhoneWidth: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { dungeonCount: 3, diagnosis: makeDiagnosis("nobody_answered", { ended: true }), elapsedS: 600 },
};

// --- Outlook from this computer's earlier searches ---

export const OutlookUsuallyFoundHealthy: Story = { args: { diagnosis: makeDiagnosis("healthy", { outlook: "usually_found" }) } };
export const OutlookSometimesFoundHealthy: Story = { args: { diagnosis: makeDiagnosis("healthy", { outlook: "sometimes_found" }) } };
export const OutlookRarelyFoundHealthy: Story = { args: { diagnosis: makeDiagnosis("healthy", { outlook: "rarely_found" }) } };
export const OutlookUsuallyFoundOthersGrouping: Story = { args: { diagnosis: makeDiagnosis("others_grouping", { outlook: "usually_found" }), elapsedS: 90 } };
export const OutlookSometimesFoundOthersGrouping: Story = { args: { diagnosis: makeDiagnosis("others_grouping", { outlook: "sometimes_found" }), elapsedS: 90 } };
export const OutlookRarelyFoundOthersGrouping: Story = { args: { diagnosis: makeDiagnosis("others_grouping", { outlook: "rarely_found" }), elapsedS: 90 } };
export const OutlookRarelyFoundNoTank: Story = { args: { diagnosis: makeDiagnosis("no_tank", { outlook: "rarely_found" }), elapsedS: 150 } };
export const OutlookSometimesFoundNobodyAnswered: Story = { args: { diagnosis: makeDiagnosis("nobody_answered", { outlook: "sometimes_found" }), elapsedS: 150 } };
/** The server guards this; if it happens anyway the clause is dropped, so this looks the same as no outlook. */
export const OutlookUsuallyFoundWithProblemDropped: Story = { args: { diagnosis: makeDiagnosis("no_tank", { outlook: "usually_found" }), elapsedS: 150 } };
/** A value from a newer server is ignored without breaking the card. */
export const OutlookUnknownValueIgnored: Story = { args: { diagnosis: makeDiagnosis("healthy", { outlook: "always_found" }) } };
export const OutlookNull: Story = { args: { diagnosis: makeDiagnosis("healthy", { outlook: null }) } };
export const OutlookPhoneWidth: Story = {
  globals: { viewport: { value: "mobile1", isRotated: false } },
  args: { dungeonCount: 3, diagnosis: makeDiagnosis("others_grouping", { outlook: "rarely_found", attempts: makeAttempts({ takenSeen: 40 }) }), elapsedS: 200 },
};
