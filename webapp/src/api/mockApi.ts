import { ApiError } from "./errors";
import { dungeonsFor, idleSearch, makeAddon, makeCharacter, makeDiagnosis, makeState } from "./fixtures";
import type { AddonInfo, AddonStatus, Api, AppState, ConnectionStatus, Settings } from "./types";

export interface MockApiOptions {
  initial?: AppState;
  /** Artificial delay per call, to see loading states. */
  latencyMs?: number;
  /** Make loopback-only routes answer 403, as they do for a phone. */
  remote?: boolean;
  /** Start the stream in a given status. Default "live". */
  status?: ConnectionStatus;
  /** The addon state the stories start from. Default: a saved folder with nothing installed. */
  addon?: AddonInfo;
  /** Make every install fail with this server error (code and plain message). */
  addonInstallError?: { status: number; code: string; message: string };
}

export interface MockApi extends Api {
  /** Addon calls in order, e.g. `["folder:/path", "install"]`, so a story can check the order. */
  addonLog: string[];
  /** Push a new state to subscribers, as the server's event stream would. */
  emit(state: AppState): void;
  current(): AppState;
}

const wait = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

/** Status after a folder change: a plain version compare (the mock has no file bytes, so `files_differ` is only kept). */
function statusOf(installed: AddonInfo["installed"], bundled: string, previous: AddonStatus): AddonStatus {
  if (installed.state === "missing") return "not_installed";
  if (installed.state === "unreadable" || installed.version === null) return "unknown_version";
  const a = installed.version.split(".").map(Number);
  const b = bundled.split(".").map(Number);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    if ((a[i] ?? 0) !== (b[i] ?? 0)) return (a[i] ?? 0) > (b[i] ?? 0) ? "installed_newer" : "update_available";
  }
  return previous === "files_differ" ? "files_differ" : "up_to_date";
}

/** In-memory Api for stories and tests. Codes: `WFT1:...` parses, `WFT2:` is an unknown version, `WFT1:BAD` fails the check. */
export function createMockApi(opts: MockApiOptions = {}): MockApi {
  let state = opts.initial ?? makeState();
  const listeners = new Set<(s: AppState) => void>();
  const delay = () => wait(opts.latencyMs ?? 0);
  const push = (next: AppState) => {
    state = next;
    listeners.forEach((l) => l(state));
  };
  let addon = opts.addon ?? makeAddon();
  const addonLog: string[] = [];
  const loopbackOnly = () => {
    if (opts.remote) throw new ApiError(403, "forbidden", "This can only be changed on the desktop.");
  };

  return {
    addonLog,
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
        search: { ...idleSearch(), status: "searching", dungeons, startedAt: state.serverNow, stageClockStartedAt: state.serverNow, searching: null, topics: dungeons.length, diagnosis: makeDiagnosis("warming_up") },
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
      if ("lan_mode" in patch || "lan_address" in patch || "trusted_peers" in patch) loopbackOnly();
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
    async getAddon() {
      await delay();
      loopbackOnly();
      return addon;
    },
    async setAddonFolder(folder) {
      await delay();
      loopbackOnly();
      addonLog.push(`folder:${folder}`);
      const path = folder.trim();
      if (!/[\\/]interface[\\/]addons$/i.test(path)) {
        throw new ApiError(400, "addon_folder_invalid", `${path} is not a World of Warcraft AddOns folder. It should end in Interface/AddOns.`);
      }
      const known = addon.status !== "no_folder" && addon.status !== "folder_invalid";
      const installed = known ? addon.installed : { state: "missing" as const, version: null };
      addon = { ...addon, folder: path, folder_source: "saved", installed, status: statusOf(installed, addon.bundled_version, addon.status) };
      return addon;
    },
    async installAddon() {
      await delay();
      loopbackOnly();
      addonLog.push("install");
      if (addon.folder === null || addon.folder_source !== "saved") {
        throw new ApiError(409, "addon_folder_unset", "No AddOns folder is saved yet.");
      }
      if (opts.addonInstallError) {
        const e = opts.addonInstallError;
        throw new ApiError(e.status, e.code, e.message);
      }
      addon = { ...addon, installed: { state: "present", version: addon.bundled_version }, status: "up_to_date" };
      return addon;
    },
    async clearHistory() {
      await delay();
      loopbackOnly();
    },
    async resetToken() {
      await delay();
      loopbackOnly();
    },
  };
}
