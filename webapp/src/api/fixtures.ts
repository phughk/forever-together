import type {
  AddonInfo,
  AddonStatus,
  AppState,
  CharacterView,
  AttemptsView,
  DiagnosisView,
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
  { slug: "ragefire-chasm", name: "Ragefire Chasm", levelMin: 13, levelMax: 18, minLevel: 13, factionHint: "horde", zone: "Orgrimmar" },
  { slug: "wailing-caverns", name: "Wailing Caverns", levelMin: 15, levelMax: 25, minLevel: 15, factionHint: "none", zone: "The Barrens" },
  { slug: "the-deadmines", name: "The Deadmines", levelMin: 17, levelMax: 26, minLevel: 15, factionHint: "alliance", zone: "Westfall" },
  { slug: "shadowfang-keep", name: "Shadowfang Keep", levelMin: 22, levelMax: 30, minLevel: 18, factionHint: "none", zone: "Silverpine Forest" },
  { slug: "blackfathom-deeps", name: "Blackfathom Deeps", levelMin: 24, levelMax: 32, minLevel: 20, factionHint: "none", zone: "Ashenvale" },
  { slug: "the-stockade", name: "The Stockade", levelMin: 24, levelMax: 32, minLevel: 22, factionHint: "alliance", zone: "Stormwind City" },
  { slug: "gnomeregan", name: "Gnomeregan", levelMin: 29, levelMax: 38, minLevel: 24, factionHint: "alliance", zone: "Dun Morogh" },
  { slug: "razorfen-kraul", name: "Razorfen Kraul", levelMin: 29, levelMax: 38, minLevel: 25, factionHint: "none", zone: "The Barrens" },
  { slug: "scarlet-monastery", name: "Scarlet Monastery", levelMin: 34, levelMax: 45, minLevel: 30, factionHint: "none", zone: "Tirisfal Glades" },
  { slug: "razorfen-downs", name: "Razorfen Downs", levelMin: 37, levelMax: 46, minLevel: 35, factionHint: "none", zone: "The Barrens" },
  { slug: "uldaman", name: "Uldaman", levelMin: 41, levelMax: 51, minLevel: 35, factionHint: "none", zone: "Badlands" },
  { slug: "maraudon", name: "Maraudon", levelMin: 46, levelMax: 55, minLevel: 40, factionHint: "none", zone: "Desolace" },
  { slug: "sunken-temple", name: "The Temple of Atal'Hakkar", levelMin: 50, levelMax: 56, minLevel: 45, factionHint: "none", zone: "Swamp of Sorrows" },
  // No known minimum level: only the level range limits the recommendation.
  { slug: "hall-of-thanes", name: "Hall of Thanes", levelMin: 13, levelMax: 18, minLevel: null, factionHint: "none", zone: "Ironforge" },
];

export function dungeonsFor(level: number, zone: string): DungeonsResponse {
  const recommended = DUNGEON_TABLE.filter(
    (d) => (d.minLevel === null || d.minLevel <= level) && d.levelMin - 3 <= level && level <= d.levelMax + 3,
  )
    .sort((a, b) => Number(b.zone === zone) - Number(a.zone === zone))
    .map((d) => d.slug);
  return { dungeons: DUNGEON_TABLE, recommended };
}

/** `n` dungeons (the table repeated with new slugs) to prove the list scrolls inside its own region. The first four are recommended. */
export function manyDungeons(n: number): DungeonsResponse {
  const dungeons = Array.from({ length: n }, (_, i) => {
    const base = DUNGEON_TABLE[i % DUNGEON_TABLE.length]!;
    return i < DUNGEON_TABLE.length ? base : { ...base, slug: `${base.slug}-${i}`, name: `${base.name} (${i})` };
  });
  return { dungeons, recommended: dungeons.slice(0, 4).map((d) => d.slug) };
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
    realm: "15",
    realmName: "PvE",
    region: 3,
    regionName: "EU",
    zone: "Westfall",
    zoneKnown: true,
    createdAt: NOW - 2 * MIN,
    ...over,
  };
}

export function makeSettings(over: Partial<Settings> = {}): Settings {
  return { lan_mode: false, lan_address: null, known_peers_size: 100, trusted_peers: [], ...over };
}

/** Placeholder AddOns folders for stories. Never a real user path. */
export const MAC_ADDONS = "/Applications/World of Warcraft/_forever_/Interface/AddOns";
export const WIN_ADDONS = "C:\\Program Files (x86)\\World of Warcraft\\_forever_\\Interface\\AddOns";

/** The addon state for a status, with the versions the brief's table uses (this app 0.2.0 unless overridden). */
export function makeAddon(status: AddonStatus = "not_installed", over: Partial<AddonInfo> = {}): AddonInfo {
  const bundled_version = over.bundled_version ?? "0.2.0";
  const base: AddonInfo = {
    bundled_version,
    folder: MAC_ADDONS,
    folder_source: "saved",
    candidates: [MAC_ADDONS],
    installed: { state: "missing", version: null },
    status,
  };
  switch (status) {
    case "no_folder":
      return { ...base, folder: null, folder_source: null, candidates: [], ...over };
    case "folder_invalid":
      return { ...base, folder: "/Applications/World of Warcraft/_classic_/Interface", candidates: [], ...over };
    case "not_installed":
      return { ...base, ...over };
    case "up_to_date":
    case "files_differ":
      return { ...base, installed: { state: "present", version: bundled_version }, ...over };
    case "update_available":
      return { ...base, installed: { state: "present", version: "0.1.0" }, ...over };
    case "installed_newer":
      return { ...base, installed: { state: "present", version: "0.3.0" }, ...over };
    case "unknown_version":
      return { ...base, installed: { state: "unreadable", version: null }, ...over };
  }
}

export function idleSearch(): SearchView {
  return { status: "idle", dungeons: [], startedAt: null, stage: 0, stageClockStartedAt: null, searching: null, topics: 0 };
}

/** Attempt counters for stories; override any field. */
export function makeAttempts(over: Partial<AttemptsView> = {}): AttemptsView {
  return { invitesSent: 0, acceptsGot: 0, invitesGot: 0, takenSeen: 0, drops: 0, declines: 0, ...over };
}

/** A diagnosis for stories and the mock. Counts default to a plausible mix for the situation. */
export function makeDiagnosis(situation: string, over: Partial<DiagnosisView> = {}): DiagnosisView {
  const seen = (
    {
      warming_up: { players: 0, tanks: 0, healers: 0, damage: 0 },
      not_connected: { players: 0, tanks: 0, healers: 0, damage: 0 },
      alone: { players: 0, tanks: 0, healers: 0, damage: 0 },
      no_tank: { players: 8, tanks: 0, healers: 2, damage: 6 },
      no_healer: { players: 7, tanks: 2, healers: 0, damage: 5 },
      short_on_damage: { players: 4, tanks: 1, healers: 2, damage: 1 },
      healthy: { players: 13, tanks: 2, healers: 2, damage: 9 },
      others_grouping: { players: 3, tanks: 0, healers: 1, damage: 2 },
      nobody_answered: { players: 13, tanks: 2, healers: 2, damage: 9 },
    } as Record<string, DiagnosisView["seen"]>
  )[situation] ?? { players: 6, tanks: 1, healers: 1, damage: 4 };
  return {
    situation,
    ended: false,
    seen,
    ownRole: "damage",
    sampleFull: false,
    networkPeers: situation === "not_connected" ? 0 : 5,
    ...(situation === "others_grouping" ? { attempts: makeAttempts({ takenSeen: 10 }) } : {}),
    ...(situation === "nobody_answered" ? { attempts: makeAttempts({ invitesSent: 6 }) } : {}),
    ...over,
  };
}

function derived(view: SearchView): DiagnosisView {
  const ended = view.status === "limit_reached";
  if (view.searching === null) return makeDiagnosis("warming_up", { ended });
  if (view.searching <= 1) return makeDiagnosis("alone", { ended });
  // The mock cycles the three outlook values by how busy the topic is, so each can be seen in the mock API.
  const outlook = view.searching >= 10 ? "usually_found" : view.searching >= 4 ? "sometimes_found" : "rarely_found";
  return makeDiagnosis("healthy", { ended, outlook: ended ? null : outlook, seen: { players: view.searching, tanks: 2, healers: 2, damage: Math.max(0, view.searching - 4) } });
}

/** A search that began `elapsedS` seconds ago on the shared 60 s schedule. */
export function searchingFor(elapsedS: number, over: Partial<SearchView> = {}): SearchView {
  const stage = Math.min(4, Math.floor(elapsedS / 60)) as SearchView["stage"];
  const view: SearchView = {
    status: "searching",
    dungeons: ["the-deadmines"],
    startedAt: NOW - elapsedS * 1000,
    stage,
    stageClockStartedAt: NOW - elapsedS * 1000,
    searching: 14,
    topics: 1,
    diagnosis: null,
    ...over,
  };
  return { ...view, diagnosis: "diagnosis" in over ? over.diagnosis : derived(view) };
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
