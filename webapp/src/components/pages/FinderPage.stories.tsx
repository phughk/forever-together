import type { Meta, StoryObj } from "@storybook/react-vite";
import { ApiError, describeCodeError } from "../../api/errors";
import {
  MIN,
  NOW,
  dungeonsFor,
  idleSearch,
  makeAttempts,
  makeCharacter,
  makeDiagnosis,
  makeMember,
  makeParty,
  makeState,
  manyDungeons,
  searchingFor,
} from "../../api/fixtures";
import { FinderPage } from "./FinderPage";

const character = makeCharacter();
const rec = dungeonsFor(character.level, character.zone);
const three = ["the-deadmines", "wailing-caverns", "shadowfang-keep"];

const meta = {
  title: "Pages/Finder",
  component: FinderPage,
  parameters: { layout: "fullscreen" },
  args: {
    state: makeState(),
    dungeons: rec,
    now: NOW,
    status: "live",
    codeError: null,
    onNavigate: () => {},
    onSubmitCode: () => {},
    onForgetCharacter: () => {},
    onStartSearch: () => {},
    onStopSearch: () => {},
    onConfirmParty: () => {},
    onDeclineParty: () => {},
  },
} satisfies Meta<typeof FinderPage>;
export default meta;
type Story = StoryObj<typeof meta>;

const phone = { globals: { viewport: { value: "mobile1", isRotated: false } } } as const;

// ---------------------------------------------------------------- first run

/** No character code yet: steps 1 to 3, and the greyed step 4. */
export const FirstRun: Story = { args: { state: makeState({ character: null }), dungeons: dungeonsFor(0, "") } };

/** The addon and the app disagree about the code version. */
export const UnknownCodeVersion: Story = {
  args: {
    state: makeState({ character: null }),
    dungeons: dungeonsFor(0, ""),
    codeError: describeCodeError(new ApiError(422, "UnknownVersion", "unknown version 2")),
  },
};

export const FirstRunOnPhone: Story = { ...phone, args: { ...FirstRun.args } };

// --------------------------------------------------------------------- idle

/** One character line, then the dungeon list. Tapping a row starts the search. */
export const Idle: Story = {};
export const IdleOneDungeon: Story = { args: { dungeons: { dungeons: rec.dungeons.slice(0, 1), recommended: rec.dungeons.slice(0, 1).map((d) => d.slug) } } };
export const IdleThirtyDungeons: Story = { args: { dungeons: manyDungeons(30) } };
export const IdleAfterStop: Story = { args: { state: makeState({ search: idleSearch() }) } };
export const IdleOnPhone: Story = { ...phone, args: {} };
export const IdleThirtyDungeonsOnPhone: Story = { ...phone, args: { dungeons: manyDungeons(30) } };

/** A code made 95 minutes ago. */
export const StaleCode: Story = { args: { state: makeState({ character: makeCharacter({ createdAt: NOW - 95 * MIN }) }) } };

/** The zone is not in the table, so no dungeon is ordered by nearness. */
export const UnknownZone: Story = {
  args: {
    state: makeState({ character: makeCharacter({ zone: "Dire Maul Outskirts", zoneKnown: false }) }),
    dungeons: dungeonsFor(24, "Dire Maul Outskirts"),
  },
};

/** A level above every dungeon range, so nothing is recommended. */
export const NoRecommendations: Story = {
  args: {
    state: makeState({ character: makeCharacter({ level: 60, itemLevel: 180, zone: "Orgrimmar", faction: "horde" }) }),
    dungeons: dungeonsFor(60, "Orgrimmar"),
  },
};

/** The longest names the code allows: 24 bytes each. */
export const LongFirstNameAndSurname: Story = {
  args: { state: makeState({ character: makeCharacter({ firstName: "Bartholomewmaximilianx", surname: "Featherstonehaughcholmo" }) }) },
};
export const LongNamesOnPhone: Story = { ...phone, args: { ...LongFirstNameAndSurname.args } };

// ---------------------------------------------------------------- searching

export const SearchingBusyTopic: Story = {
  args: { state: makeState({ search: searchingFor(12, { searching: 312, topics: 3, dungeons: three }) }) },
};
export const SearchingNotConnected: Story = {
  args: { state: makeState({ search: searchingFor(40, { searching: 0, diagnosis: makeDiagnosis("not_connected") }) }) },
};
export const SearchingNoTank: Story = {
  args: { state: makeState({ search: searchingFor(130, { searching: 8, diagnosis: makeDiagnosis("no_tank") }) }) },
};
export const SearchingNoTankRarelyFound: Story = {
  args: { state: makeState({ search: searchingFor(130, { searching: 8, diagnosis: makeDiagnosis("no_tank", { outlook: "rarely_found" }) }) }) },
};
export const SearchingOthersGrouping: Story = {
  args: { state: makeState({ search: searchingFor(100, { searching: 3, diagnosis: makeDiagnosis("others_grouping") }) }) },
};
export const SearchingOthersGroupingSometimesFound: Story = {
  args: { state: makeState({ search: searchingFor(100, { searching: 3, diagnosis: makeDiagnosis("others_grouping", { outlook: "sometimes_found" }) }) }) },
};
export const SearchingNobodyAnswered: Story = {
  args: { state: makeState({ search: searchingFor(130, { diagnosis: makeDiagnosis("nobody_answered") }) }) },
};
/** The schedule is at its last step and almost nobody is looking. */
export const SearchingQuietTopicAtLastStep: Story = {
  args: { state: makeState({ search: searchingFor(385, { searching: 1, dungeons: ["blackfathom-deeps"] }) }) },
};
/** An older server sends no diagnosis: only a rough count of players is shown. */
export const SearchingOldServer: Story = { args: { state: makeState({ search: searchingFor(75, { diagnosis: null }) }) } };
export const SearchingOldServerCounting: Story = { args: { state: makeState({ search: searchingFor(5, { searching: null, diagnosis: null }) }) } };
export const SearchingOnPhone: Story = { ...phone, args: { state: makeState({ search: searchingFor(130, { searching: 28 }) }) } };
export const SearchingNoTankOnPhone: Story = { ...phone, args: { ...SearchingNoTank.args } };
export const ConnectionLost: Story = { args: { status: "reconnecting", state: makeState({ search: searchingFor(50) }) } };

// -------------------------------------------------------------------- ended

const ended = { status: "limit_reached", stage: 4 } as const;
export const EndedOthersGrouping: Story = {
  args: {
    state: makeState({
      search: searchingFor(600, { ...ended, searching: 3, diagnosis: makeDiagnosis("others_grouping", { ended: true, attempts: makeAttempts({ takenSeen: 25 }) }) }),
    }),
  },
};
export const EndedNobodyAnsweredSeveralDungeons: Story = {
  args: {
    state: makeState({
      search: searchingFor(600, {
        ...ended,
        topics: 3,
        dungeons: three,
        diagnosis: makeDiagnosis("nobody_answered", { ended: true, attempts: makeAttempts({ invitesSent: 18 }) }),
      }),
    }),
  },
};
export const EndedNoTankSeveralDungeons: Story = {
  args: {
    state: makeState({
      search: searchingFor(600, {
        ...ended,
        topics: 3,
        dungeons: three,
        diagnosis: makeDiagnosis("no_tank", { ended: true, sampleFull: true, seen: { players: 64, tanks: 0, healers: 12, damage: 52 } }),
      }),
    }),
  },
};
export const EndedAlone: Story = { args: { state: makeState({ search: searchingFor(600, { ...ended, searching: 1, diagnosis: makeDiagnosis("alone", { ended: true }) }) }) } };
/** An older server sends no diagnosis. */
export const EndedOldServer: Story = { args: { state: makeState({ search: searchingFor(600, { ...ended, diagnosis: null }) }) } };
export const EndedOnPhone: Story = { ...phone, args: { ...EndedNoTankSeveralDungeons.args } };

// ------------------------------------------------------------ party found

/** A party formed and waits for answers. Members that accepted show a tick; names stay hidden until all five accept. */
const found = (accepted: number) => ({
  args: {
    state: makeState({
      search: searchingFor(75),
      party: makeParty({}, [0, 1, 2, 3, 4].map((i) => ({ confirmed: i < accepted }))),
    }),
  },
});
/** You are member 0. With 0 accepted you have not answered; with 1 or more you have. */
export const PartyFoundNoneAccepted: Story = found(0);
export const PartyFoundOneAccepted: Story = found(1);
export const PartyFoundTwoAccepted: Story = found(2);
export const PartyFoundThreeAccepted: Story = found(3);
export const PartyFoundFourAccepted: Story = found(4);
export const PartyFoundFiveAccepted: Story = found(5);
/** Four accepted, one is still ready but has not answered. */
export const PartyFoundFourAcceptedOneWaiting: Story = {
  args: {
    state: makeState({
      search: searchingFor(75),
      party: makeParty({}, [{ confirmed: true }, { confirmed: true }, { confirmed: true }, { confirmed: true }, { ready: true, confirmed: false }]),
    }),
  },
};
export const PartyFoundOnPhone: Story = { ...phone, args: { ...found(0).args } };
export const PartyFoundOnPhoneTwoAccepted: Story = { ...phone, args: { ...found(2).args } };

// ------------------------------------------------------------ party final

const finalParty = {
  ...makeParty({ phase: "final", confirmDeadline: null }),
  members: [0, 1, 2, 3, 4].map((i) => makeMember(i, { confirmed: true })),
};
export const PartyFinal: Story = { args: { state: makeState({ party: finalParty }) } };
export const PartyFinalOnPhone: Story = { ...phone, args: { state: makeState({ party: finalParty }) } };
export const PartyFinalLongNames: Story = {
  args: {
    state: makeState({
      party: {
        ...finalParty,
        members: finalParty.members.map((m, i) => ({ ...m, firstName: "Bartholomewmaximilianx", surname: `Featherstonehaughcholmo${i}`.slice(0, 24) })),
      },
    }),
  },
};

// ---------------------------------------------------------- party dropped

const dropped = (reason: "declined" | "timeout" | "unreachable") => ({
  args: {
    state: makeState({ search: searchingFor(5), party: makeParty({ phase: "dropped", dropReason: reason, confirmDeadline: null }) }),
    onDismissParty: () => {},
  },
});
export const PartyDroppedDeclined: Story = dropped("declined");
export const PartyDroppedTimeout: Story = dropped("timeout");
export const PartyDroppedUnreachable: Story = dropped("unreachable");
export const PartyDroppedOnPhone: Story = { ...phone, args: { ...dropped("unreachable").args } };
