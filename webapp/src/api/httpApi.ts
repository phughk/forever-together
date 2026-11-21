import { ApiError } from "./errors";
import { openEventStream } from "./eventStream";
import type { TokenStore } from "./token";
import type { Api, AppState, CharacterView, DungeonsResponse, PairingInfo, Settings } from "./types";

export interface HttpApiOptions {
  tokens: TokenStore;
  baseUrl?: string;
  fetchImpl?: typeof fetch;
}

export function createHttpApi({ tokens, baseUrl = "", fetchImpl }: HttpApiOptions): Api {
  const doFetch = fetchImpl ?? fetch.bind(globalThis);

  async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const token = tokens.get();
    const res = await doFetch(`${baseUrl}${path}`, {
      method,
      headers: {
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(body !== undefined ? { "Content-Type": "application/json" } : {}),
      },
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) {
      let code = `http_${res.status}`;
      let message = res.statusText;
      try {
        const err = (await res.json()) as { error?: string; message?: string };
        code = err.error ?? code;
        message = err.message ?? message;
      } catch {
        // Non-JSON error body, keep the defaults.
      }
      throw new ApiError(res.status, code, message);
    }
    const text = await res.text();
    return (text ? JSON.parse(text) : undefined) as T;
  }

  return {
    getState: () => request<AppState>("GET", "/api/state"),
    subscribe({ onState, onStatus }) {
      return openEventStream({
        url: `${baseUrl}/api/events`,
        getToken: () => tokens.get(),
        fetchImpl: doFetch,
        onStatus,
        onOpen: () => {
          // A snapshot after every (re)connect covers anything missed while the link was down.
          request<AppState>("GET", "/api/state")
            .then(onState)
            .catch((e: unknown) => {
              if (e instanceof ApiError && e.status === 401) onStatus("unauthorized");
            });
        },
        onEvent: (event) => {
          if (event.type === "state") onState(event.state);
        },
      });
    },
    submitCharacter: (code) => request<CharacterView>("POST", "/api/character", { code }),
    forgetCharacter: () => request<void>("DELETE", "/api/character"),
    getDungeons: () => request<DungeonsResponse>("GET", "/api/dungeons"),
    startSearch: (dungeons) => request<void>("POST", "/api/search", { dungeons }),
    stopSearch: () => request<void>("DELETE", "/api/search"),
    confirmParty: () => request<void>("POST", "/api/party/confirm"),
    declineParty: () => request<void>("POST", "/api/party/decline"),
    getSettings: () => request<Settings>("GET", "/api/settings"),
    putSettings: (patch) => request<Settings>("PUT", "/api/settings", patch),
    getPairing: (ip) => request<PairingInfo>("GET", ip ? `/api/pairing?ip=${encodeURIComponent(ip)}` : "/api/pairing"),
    async resetToken() {
      // The desktop window receives the new token in the response and keeps working.
      const res = await request<{ token?: string } | undefined>("POST", "/api/token/reset");
      if (res?.token) tokens.set(res.token);
    },
  };
}
