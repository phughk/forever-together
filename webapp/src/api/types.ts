// Wire types for the local web API (implementation plan, "Local web API").
//
// Assumptions where the plan is silent (to be reconciled with app/src/server):
//  - JSON is camelCase, except settings keys, which use the plan's names
//    (`lan_mode`, `trusted_peers`) plus `known_peers_size`.
//  - Times are absolute server-clock milliseconds; `AppState.serverNow` lets the
//    client correct for a phone clock that differs from the desktop clock.
//  - Class and race arrive as display names, resolved by the server.
//  - `GET /api/events` is newline-delimited JSON: `{"type":"state","state":...}`
//    for each change and `{"type":"heartbeat"}` every 15 s.

export type Role = "tank" | "healer" | "damage" | "unknown";
export type Faction = "alliance" | "horde";
export type FactionHint = Faction | "both" | "none";

/** Widening stage, 0 (strict) to 4 (widest). See the plan's stage table. */
export type Stage = 0 | 1 | 2 | 3 | 4;

/** What the app understood from the pasted character code. */
export interface CharacterView {
  firstName: string;
  surname: string;
  faction: Faction;
  className: string | null;
  raceName: string | null;
  level: number;
  /** 0 when the client could not supply it. */
  itemLevel: number;
  role: Role;
  realm: string;
  /** Display name of `realm`: PvP, PvE, RP-PvP, or "realm <id>" when unknown. */
  realmName: string;
  region: number;
  /** Display name of `region`: EU, US, ... or "region <n>" when unknown. */
  regionName: string;
  /** Zone name as the client reported it, empty when unknown. */
  zone: string;
  /** False when the zone is empty or not in the dungeon table. */
  zoneKnown: boolean;
  /** Server-clock ms when the addon built the code. */
  createdAt: number;
}

export interface DungeonEntry {
  slug: string;
  name: string;
  levelMin: number;
  levelMax: number;
  /** `null`: no known minimum level. */
  minLevel: number | null;
  factionHint: FactionHint;
  zone: string;
}

export interface DungeonsResponse {
  dungeons: DungeonEntry[];
  /** Slugs in recommendation order. May be empty. */
  recommended: string[];
}

export type SearchStatus = "idle" | "searching" | "limit_reached";

/** Roles a diagnosis can name. */
export type DiagnosisRole = "tank" | "healer" | "damage";

/** What this computer did and saw of the agreement round trips, as counts. Absent from an older server. */
export interface AttemptsView {
  invitesSent: number;
  acceptsGot: number;
  invitesGot: number;
  takenSeen: number;
  drops: number;
  declines: number;
}

/** What this computer can see while searching, as counts. The webapp writes the sentences. */
export interface DiagnosisView {
  /**
   * Known codes: "warming_up", "not_connected", "alone", "no_tank", "no_healer", "short_on_damage", "healthy",
   * "others_grouping", "nobody_answered".
   * Any other string must be tolerated.
   */
  situation: string;
  /** True when the search expired: the numbers are the frozen peak over the search, not live. */
  ended: boolean;
  /** Other compatible players seen (the peak at the end). Never includes the player. */
  seen: { players: number; tanks: number; healers: number; damage: number };
  ownRole: DiagnosisRole | null;
  /** The counts are a slice of a capped sample, so they are "at least" or "about". */
  sampleFull: boolean;
  networkPeers: number;
  /** Absent from an older server: render without attempt counts. */
  attempts?: AttemptsView;
}

export interface SearchView {
  status: SearchStatus;
  dungeons: string[];
  startedAt: number | null;
  stage: Stage;
  /** Start of the stage clock. It restarts when a dropped party returns the peer to searching. */
  stageClockStartedAt: number | null;
  /** Players searching on the joined topics, as far as this peer can see. Null until known. */
  searching: number | null;
  topics: number;
  /** Absent from an older server: render the search as before. */
  diagnosis?: DiagnosisView | null;
}

export type PartyPhase = "confirming" | "final" | "dropped";
export type DropReason = "declined" | "timeout" | "unreachable";

export interface PartyMember {
  peerId: string;
  /** Present only once the party is final. Claimed by the other player, never verified. */
  firstName: string | null;
  surname: string | null;
  role: Role;
  level: number;
  itemLevel: number;
  self: boolean;
  ready: boolean;
  confirmed: boolean;
  declined: boolean;
}

export interface PartyView {
  id: string;
  phase: PartyPhase;
  dropReason: DropReason | null;
  members: PartyMember[];
  /** Server-clock ms deadline for the human Confirm click, null once decided. */
  confirmDeadline: number | null;
}

export type BootstrapSource = "remembered" | "peer_exchange" | "trusted" | "none";

export interface NetworkView {
  peers: number;
  bootstrap: BootstrapSource;
}

export interface Settings {
  lan_mode: boolean;
  /** The address the pairing link uses; null picks one automatically. */
  lan_address: string | null;
  known_peers_size: number;
  trusted_peers: string[];
}

export interface AppState {
  serverNow: number;
  character: CharacterView | null;
  search: SearchView;
  party: PartyView | null;
  network: NetworkView;
  settings: Settings;
}

export interface PairingInfo {
  url: string;
  /** The address the link uses. */
  address: string;
  /** Every private address of this computer; the first is the default. */
  addresses: string[];
}

export type ApiEvent = { type: "state"; state: AppState } | { type: "heartbeat" };

export type ConnectionStatus = "connecting" | "live" | "reconnecting" | "unauthorized";

/** Everything the UI needs from the backend. Implemented over HTTP and by the mock. */
export interface Api {
  getState(): Promise<AppState>;
  /** Start the live stream. Returns a stop function. */
  subscribe(handlers: {
    onState: (state: AppState) => void;
    onStatus: (status: ConnectionStatus) => void;
  }): () => void;
  submitCharacter(code: string): Promise<CharacterView>;
  forgetCharacter(): Promise<void>;
  getDungeons(): Promise<DungeonsResponse>;
  startSearch(dungeons: string[]): Promise<void>;
  stopSearch(): Promise<void>;
  confirmParty(): Promise<void>;
  declineParty(): Promise<void>;
  getSettings(): Promise<Settings>;
  putSettings(patch: Partial<Settings>): Promise<Settings>;
  /** `ip` picks one of the listed addresses; without it the app picks the one on the active network. */
  getPairing(ip?: string): Promise<PairingInfo>;
  resetToken(): Promise<void>;
  /** Deletes the local record of finished searches. Loopback only. */
  clearHistory(): Promise<void>;
}
