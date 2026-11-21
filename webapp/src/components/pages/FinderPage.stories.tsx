import type { Meta, StoryObj } from "@storybook/react-vite";
import { ApiError, describeCodeError } from "../../api/errors";
import {
  MIN,
  NOW,
  dungeonsFor,
  idleSearch,
  makeCharacter,
  makeMember,
  makeParty,
  makeState,
  searchingFor,
} from "../../api/fixtures";
import { FinderPage } from "./FinderPage";

const character = makeCharacter();
const rec = dungeonsFor(character.level, character.zone);

const meta = {
  title: "Pages/Finder",
  component: FinderPage,
  parameters: { layout: "fullscreen" },
  args: {
    state: makeState(),
    dungeons: rec,
    now: NOW,
    status: "live",
    selected: [],
    codeError: null,
    onNavigate: () => {},
    onSubmitCode: () => {},
    onForgetCharacter: () => {},
    onSelectionChange: () => {},
    onStartSearch: () => {},
    onStopSearch: () => {},
    onConfirmParty: () => {},
    onDeclineParty: () => {},
  },
} satisfies Meta<typeof FinderPage>;
export default meta;
type Story = StoryObj<typeof meta>;

const phone = { globals: { viewport: { value: "mobile1", isRotated: false } } } as const;

// --- Required page stories (implementation plan, item 26) ---

/** CUJ 4: nothing remembered, no character code yet. */
export const FirstRun: Story = {
  args: { state: makeState({ character: null, network: { peers: 0, bootstrap: "none" } }), dungeons: dungeonsFor(0, "") },
};

/** A code made 95 minutes ago. */
export const StaleCode: Story = {
  args: { state: makeState({ character: makeCharacter({ createdAt: NOW - 95 * MIN }) }) },
};

/** The addon and the app disagree about the code version. */
export const UnknownCodeVersion: Story = {
  args: {
    state: makeState({ character: null }),
    dungeons: dungeonsFor(0, ""),
    codeError: describeCodeError(new ApiError(422, "UnknownVersion", "unknown version 2")),
  },
};

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
  args: {
    state: makeState({ character: makeCharacter({ firstName: "Bartholomewmaximilianx", surname: "Featherstonehaughcholmo" }) }),
  },
};

/** CUJ 3: nobody else is looking, and the schedule is at its last stage. */
export const QuietTopicAtLastStage: Story = {
  args: {
    state: makeState({
      search: searchingFor(385, { searching: 1, dungeons: ["blackfathom-deeps"] }),
      network: { peers: 1, bootstrap: "trusted" },
    }),
    selected: ["blackfathom-deeps"],
  },
};

/** A party formed. Four members are Ready and confirmed, one is Ready but has not confirmed. */
export const PartyMemberReadyNotConfirmed: Story = {
  args: {
    state: makeState({
      search: searchingFor(75),
      party: makeParty({}, [{ confirmed: true }, { confirmed: true }, { confirmed: true }, { confirmed: true }, { ready: true, confirmed: false }]),
    }),
  },
};

/** Phone width, searching. */
export const PhoneWidth: Story = { ...phone, args: { state: makeState({ search: searchingFor(130, { searching: 28 }) }) } };

// --- Variations (CLAUDE.md: cover the template, not just the happy path) ---

export const ReadyToSearchOneSelected: Story = { args: { selected: ["the-deadmines"] } };
export const ReadyToSearchSeveralSelected: Story = { args: { selected: ["the-deadmines", "wailing-caverns", "shadowfang-keep"] } };

export const SearchingBusyTopic: Story = {
  args: { state: makeState({ search: searchingFor(12, { searching: 312, topics: 3, dungeons: ["the-deadmines", "wailing-caverns", "shadowfang-keep"] }) }) },
};

export const SearchEndedAtLimit: Story = {
  args: { state: makeState({ search: searchingFor(600, { status: "limit_reached", stage: 4 }) }) },
};

export const PartyAwaitingYourConfirm: Story = {
  args: { state: makeState({ search: searchingFor(75), party: makeParty() }) },
};

export const PartyFinalNamesRevealed: Story = {
  args: {
    state: makeState({
      party: {
        ...makeParty({ phase: "final", confirmDeadline: null }),
        members: [0, 1, 2, 3, 4].map((i) => makeMember(i, { confirmed: true })),
      },
    }),
  },
};

export const PartyDroppedMemberDidNotConfirm: Story = {
  args: {
    state: makeState({
      search: searchingFor(5),
      party: makeParty({ phase: "dropped", dropReason: "timeout", confirmDeadline: null }),
    }),
    onDismissParty: () => {},
  },
};

export const ConnectionLost: Story = { args: { status: "reconnecting", state: makeState({ search: searchingFor(50) }) } };

export const PhoneFirstRun: Story = { ...phone, args: { ...FirstRun.args } };
export const PhoneParty: Story = {
  ...phone,
  args: { state: makeState({ party: makeParty({}, [{ confirmed: true }, { confirmed: true }, {}, {}, { ready: false }]) }) },
};
export const PhoneLongNames: Story = { ...phone, args: { ...LongFirstNameAndSurname.args } };

export const IdleAfterStop: Story = { args: { state: makeState({ search: idleSearch() }) } };
