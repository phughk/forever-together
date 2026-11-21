import type {
  AppState,
  CharacterView,
  DungeonEntry,
  DungeonsResponse,
  PartyMember,
  PartyView,
  Role,
  SearchView,
  Settings,
} from "./types";

/** Fixed server clock used by stories so every render is deterministic. */
export const NOW = 1_800_000_000_000;
export const MIN = 60_000;

export const DUNGEON_TABLE: DungeonEntry[] = [
  { slug: "ragefire-chasm", name: "Ragefire Chasm", levelMin: 13, levelMax: 18, minLevel: 13, factionHint: "horde", zone: "Orgrimmar", status: "available" },
  { slug: "wailing-caverns", name: "Wailing Caverns", levelMin: 15, levelMax: 25, minLevel: 15, factionHint: "none", zone: "The Barrens", status: "available" },
  { slug: "the-deadmines", name: "The Deadmines", levelMin: 17, levelMax: 26, minLevel: 15, factionHint: "alliance", zone: "Westfall", status: "available" },
  { slug: "shadowfang-keep", name: "Shadowfang Keep", levelMin: 22, levelMax: 30, minLevel: 18, factionHint: "none", zone: "Silverpine Forest", status: "available" },
  { slug: "blackfathom-deeps", name: "Blackfathom Deeps", levelMin: 24, levelMax: 32, minLevel: 20, factionHint: "none", zone: "Ashenvale", status: "available" },
  { slug: "the-stockade", name: "The Stockade", levelMin: 24, levelMax: 32, minLevel: 22, factionHint: "alliance", zone: "Stormwind City", status: "available" },
  { slug: "gnomeregan", name: "Gnomeregan", levelMin: 29, levelMax: 38, minLevel: 24, factionHint: "alliance", zone: "Dun Morogh", status: "available" },
  { slug: "razorfen-kraul", name: "Razorfen Kraul", levelMin: 29, levelMax: 38, minLevel: 25, factionHint: "none", zone: "The Barrens", status: "available" },
  { slug: "scarlet-monastery", name: "Scarlet Monastery", levelMin: 34, levelMax: 45, minLevel: 30, factionHint: "none", zone: "Tirisfal Glades", status: "available" },
  { slug: "razorfen-downs", name: "Razorfen Downs", levelMin: 37, levelMax: 46, minLevel: 35, factionHint: "none", zone: "The Barrens", status: "available" },
  { slug: "uldaman", name: "Uldaman", levelMin: 41, levelMax: 51, minLevel: 35, factionHint: "none", zone: "Badlands", status: "available" },
  { slug: "maraudon", name: "Maraudon", levelMin: 46, levelMax: 55, minLevel: 40, factionHint: "none", zone: "Desolace", status: "coming_soon" },
  { slug: "sunken-temple", name: "The Temple of Atal'Hakkar", levelMin: 50, levelMax: 56, minLevel: 45, factionHint: "none", zone: "Swamp of Sorrows", status: "coming_soon" },
];

export function dungeonsFor(level: number, zone: string): DungeonsResponse {
  const recommended = DUNGEON_TABLE.filter(
    (d) => d.status === "available" && d.minLevel <= level && d.levelMin - 3 <= level && level <= d.levelMax + 3,
  )
    .sort((a, b) => Number(b.zone === zone) - Number(a.zone === zone))
    .map((d) => d.slug);
  return { dungeons: DUNGEON_TABLE, recommended };
}

export const SAMPLE_CODE =
  "WFT1:KJ2 8E4%KDJ5EDGX.EC5-D3/DX0DWEDQ0GDCEC9JCUF5$CC0/E8KE9JD7DDE40DK$D5.D";

export function makeCharacter(over: Partial<CharacterView> = {}): CharacterView {
  return {
    firstName: "Aldric",
    surname: "Stormhand",
    faction: "alliance",
    className: "Warrior",
    raceName: "Human",
    level: 24,
    itemLevel: 41,
    role: "tank",
    realm: "pve",
    region: 3,
    zone: "Westfall",
    zoneKnown: true,
    createdAt: NOW - 2 * MIN,
    ...over,
  };
}

export function makeSettings(over: Partial<Settings> = {}): Settings {
  return { lan_mode: false, known_peers_size: 100, trusted_peers: [], ...over };
}

export function idleSearch(): SearchView {
  return { status: "idle", dungeons: [], startedAt: null, stage: 0, stageClockStartedAt: null, searching: null, topics: 0 };
}

/** A search that began `elapsedS` seconds ago on the shared 60 s schedule. */
export function searchingFor(elapsedS: number, over: Partial<SearchView> = {}): SearchView {
  const stage = Math.min(4, Math.floor(elapsedS / 60)) as SearchView["stage"];
  return {
    status: "searching",
    dungeons: ["the-deadmines"],
    startedAt: NOW - elapsedS * 1000,
    stage,
    stageClockStartedAt: NOW - elapsedS * 1000,
    searching: 14,
    topics: 1,
    ...over,
  };
}

const ROLES: Role[] = ["tank", "healer", "damage", "damage", "damage"];
const NAMES: Array<[string, string]> = [
  ["Aldric", "Stormhand"],
  ["Mirelle", "Dawnwhisper"],
  ["Torvin", "Ashgrove"],
  ["Sabeth", "Kellan"],
  ["Orrin", "Vale"],
];

export function makeMember(i: number, over: Partial<PartyMember> = {}): PartyMember {
  const [firstName, surname] = NAMES[i % NAMES.length] ?? ["Unknown", "Player"];
  return {
    peerId: `peer-${i}`,
    firstName,
    surname,
    role: ROLES[i % ROLES.length] ?? "damage",
    level: 24 + (i % 3),
    itemLevel: 38 + i * 2,
    self: i === 0,
    ready: true,
    confirmed: false,
    declined: false,
    ...over,
  };
}

export function makeParty(over: Partial<PartyView> = {}, memberOver: Array<Partial<PartyMember>> = []): PartyView {
  return {
    id: "party-7f3a",
    phase: "confirming",
    dropReason: null,
    confirmDeadline: NOW + 9000,
    members: [0, 1, 2, 3, 4].map((i) => makeMember(i, { firstName: null, surname: null, ...memberOver[i] })),
    ...over,
  };
}

export function makeState(over: Partial<AppState> = {}): AppState {
  return {
    serverNow: NOW,
    character: makeCharacter(),
    search: idleSearch(),
    party: null,
    network: { peers: 5, bootstrap: "remembered" },
    settings: makeSettings(),
    ...over,
  };
}
