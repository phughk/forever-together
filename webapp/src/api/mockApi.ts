import { ApiError } from "./errors";
import { dungeonsFor, idleSearch, makeCharacter, makeState } from "./fixtures";
import type { Api, AppState, ConnectionStatus, Settings } from "./types";

export interface MockApiOptions {
  initial?: AppState;
  /** Artificial delay per call, to see loading states. */
  latencyMs?: number;
  /** Make loopback-only routes answer 403, as they do for a phone. */
  remote?: boolean;
  /** Start the stream in a given status. Default "live". */
  status?: ConnectionStatus;
}

export interface MockApi extends Api {
  /** Push a new state to subscribers, as the server's event stream would. */
  emit(state: AppState): void;
  current(): AppState;
}

const wait = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

/** In-memory Api for stories and tests. Codes: `WFT1:...` parses, `WFT2:` is an unknown version, `WFT1:BAD` fails the check. */
export function createMockApi(opts: MockApiOptions = {}): MockApi {
  let state = opts.initial ?? makeState();
  const listeners = new Set<(s: AppState) => void>();
  const delay = () => wait(opts.latencyMs ?? 0);
  const push = (next: AppState) => {
    state = next;
    listeners.forEach((l) => l(state));
  };
  const loopbackOnly = () => {
    if (opts.remote) throw new ApiError(403, "forbidden", "This can only be changed on the desktop.");
  };

  return {
    emit: push,
    current: () => state,
    async getState() {
      await delay();
      return state;
    },
    subscribe({ onState, onStatus }) {
      listeners.add(onState);
      onStatus(opts.status ?? "live");
      onState(state);
      return () => listeners.delete(onState);
    },
    async submitCharacter(code) {
      await delay();
      const text = code.trim();
      if (text.startsWith("WFT1:BAD")) throw new ApiError(422, "BadCheck", "check mismatch");
      if (/^WFT\d+:/.test(text) && !text.startsWith("WFT1:")) throw new ApiError(422, "UnknownVersion", "unknown version");
      if (!text.startsWith("WFT1:")) throw new ApiError(422, "BadText", "not a code");
      const character = makeCharacter();
      push({ ...state, character });
      return character;
    },
    async forgetCharacter() {
      await delay();
      push({ ...state, character: null, search: idleSearch() });
    },
    async getDungeons() {
      await delay();
      return dungeonsFor(state.character?.level ?? 0, state.character?.zone ?? "");
    },
    async startSearch(dungeons) {
      await delay();
      push({
        ...state,
        search: { ...idleSearch(), status: "searching", dungeons, startedAt: state.serverNow, stageClockStartedAt: state.serverNow, searching: 1, topics: dungeons.length },
      });
    },
    async stopSearch() {
      await delay();
      push({ ...state, search: idleSearch() });
    },
    async confirmParty() {
      await delay();
      if (!state.party) return;
      const members = state.party.members.map((m) => (m.self ? { ...m, confirmed: true } : m));
      push({ ...state, party: { ...state.party, members } });
    },
    async declineParty() {
      await delay();
      push({ ...state, party: null, search: { ...state.search, stage: 0 } });
    },
    async getSettings() {
      await delay();
      return state.settings;
    },
    async putSettings(patch) {
      await delay();
      if ("lan_mode" in patch || "trusted_peers" in patch) loopbackOnly();
      const settings: Settings = { ...state.settings, ...patch };
      push({ ...state, settings });
      return settings;
    },
    async getPairing(ip) {
      await delay();
      loopbackOnly();
      const addresses = ["192.168.1.23", "10.0.0.7"];
      const address = ip && addresses.includes(ip) ? ip : "192.168.1.23";
      return {
        url: `http://${address}:41873/#token=3f9a1c0be27d44a8b6d15e90c7a2f8e14b0d9c63a5e7f1280b4d6c9a1e3f5072`,
        address,
        addresses,
      };
    },
    async resetToken() {
      await delay();
      loopbackOnly();
    },
  };
}
