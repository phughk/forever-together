import type { Meta, StoryObj } from "@storybook/react-vite";
import { makeAttempts, makeDiagnosis } from "../../api/fixtures";
import { expect, within } from "storybook/test";
import { HEDGE_TIP, DETAIL_MAX_CHARS, HEADLINE_MAX_CHARS, OUTLOOK_MAX_CHARS, diagnosisCopy } from "../../lib/diagnosisCopy";
import { Notice } from "../atoms/Notice";
import { INFO_TIP_MAX_CHARS, InfoTip } from "../atoms/InfoTip";
import { DiagnosisSummary } from "./DiagnosisSummary";

const meta = {
  title: "Molecules/DiagnosisSummary",
  component: DiagnosisSummary,
  parameters: {
    docs: {
      description: {
        component: [
          "Tells the player what is seen from here while a search runs, and in a summary when it ended.",
          "The backend sends a situation code and counts; the sentences are written in `lib/diagnosisCopy.ts`.",
          "",
          "Layout: a one-sentence consequence headline with the hedge in one info tip beside it, the role row (three roles against what a party needs, your own role pre-filled), short chips, and the detail and outlook sentences inside Details.",
          "Grey means not seen from here, never red, so \"none seen\" does not read as \"none exist\". The label \"Seen from here\" and the `~` before rough counts are always visible.",
          "",
          "Content limits: the headline is one sentence of at most 90 characters, the detail one sentence of at most 110, the outlook at most 110.",
          "Several dungeons add only the count (\"in 3 dungeons\"), never the names.",
          "Wording rules: claims are about what is seen from here, numbers are rough, nothing promises a group, and nothing reassures when a role is missing or the app is not connected.",
          "Chips (\"Asked 6 players to join, none replied\", \"Others are forming groups\") appear only when the server sends attempts or for that situation. An unknown situation code gets a neutral sentence.",
          "",
          "Outlook: inside Details, a quiet line says how the player's own earlier searches went (usually, sometimes or rarely found).",
          "It is history shown as a hedged clause, never as data: no numbers, percentages, counts or dates, and it never promises a group.",
          "`usually_found` next to a live problem (no tank, no healer, short on damage, nobody replied, alone, not connected, warming up) is dropped, and so is an unknown value. It is never shown for an ended search.",
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

// --- Outlook from the player's earlier searches ---

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

// --- Length and layout checks ---

const LONG = "A headline this long can never come from the copy, which stays within ninety characters; it is here only to show the layout holds. ".repeat(3).slice(0, 340);
/** Layout stress test: a 40-character and a 340-character headline in the same markup, with the info tip beside. */
export const HeadlineLengthStress: Story = {
  render: () => (
    <div className="ft-diagnosis">
      {["No tank seen, so a party can't form yet.", LONG].map((h) => (
        <Notice key={h.length} tone="warn">
          <div className="ft-diagnosis__headline">
            <span>{h}</span>
            <InfoTip label="About these counts">{HEDGE_TIP}</InfoTip>
          </div>
        </Notice>
      ))}
    </div>
  ),
};

const SITUATIONS = ["warming_up", "not_connected", "alone", "no_tank", "no_healer", "short_on_damage", "others_grouping", "nobody_answered", "healthy", "needs_a_new_code"];
/** Every sentence stays within its limit, and the hedge within the info tip cap. */
export const CopyWithinLimits: Story = {
  play: async ({ canvasElement }) => {
    await expect(within(canvasElement).getByText(/Seen from here/)).toBeVisible();
    await expect(HEDGE_TIP.length).toBeLessThanOrEqual(INFO_TIP_MAX_CHARS);
    for (const situation of SITUATIONS) {
      for (const ended of [false, true]) {
        for (const outlook of ["usually_found", "sometimes_found", "rarely_found"]) {
          const c = diagnosisCopy(makeDiagnosis(situation, { ended, outlook, ownRole: "healer", sampleFull: true }), 600, 12);
          await expect(c.headline.length).toBeLessThanOrEqual(HEADLINE_MAX_CHARS);
          await expect((c.detail ?? "").length).toBeLessThanOrEqual(DETAIL_MAX_CHARS);
          await expect((c.outlook ?? "").length).toBeLessThanOrEqual(OUTLOOK_MAX_CHARS);
        }
      }
    }
  },
};

/** A full sample with your own role (damage) pre-filled: rough counts get a `~`. */
export const FullSamplePartialRoles: Story = {
  args: { diagnosis: makeDiagnosis("short_on_damage", { sampleFull: true, seen: { players: 40, tanks: 9, healers: 11, damage: 0 } }) },
};
