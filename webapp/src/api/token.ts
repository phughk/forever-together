const STORAGE_KEY = "wowft.token";

export interface TokenStore {
  get(): string | null;
  set(token: string): void;
  clear(): void;
}

function safe<T>(fn: () => T, fallback: T): T {
  try {
    return fn();
  } catch {
    return fallback;
  }
}

/** Pull `token=...` out of a URL hash such as `#token=abc`. */
export function parseTokenFromHash(hash: string): string | null {
  const params = new URLSearchParams(hash.replace(/^#/, ""));
  const token = params.get("token");
  return token && token.length > 0 ? token : null;
}

export const localStorageTokenStore: TokenStore = {
  get: () => safe(() => localStorage.getItem(STORAGE_KEY), null),
  set: (token) => safe(() => localStorage.setItem(STORAGE_KEY, token), undefined),
  clear: () => safe(() => localStorage.removeItem(STORAGE_KEY), undefined),
};

export function memoryTokenStore(initial: string | null = null): TokenStore {
  let token = initial;
  return {
    get: () => token,
    set: (t) => {
      token = t;
    },
    clear: () => {
      token = null;
    },
  };
}

/**
 * Take the token from the address bar (pairing QR or desktop window), save it
 * for reuse and remove it from the address bar so it is not left in history.
 * Falls back to the stored token when the hash has none.
 */
export function adoptToken(
  store: TokenStore,
  loc: Pick<Location, "hash" | "pathname" | "search"> = window.location,
  history: Pick<History, "replaceState"> = window.history,
): string | null {
  const fromHash = parseTokenFromHash(loc.hash);
  if (fromHash) {
    store.set(fromHash);
    safe(() => history.replaceState(null, "", loc.pathname + loc.search), undefined);
    return fromHash;
  }
  return store.get();
}
