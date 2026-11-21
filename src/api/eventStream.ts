import type { ApiEvent, ConnectionStatus } from "./types";

export interface EventStreamOptions {
  url: string;
  getToken: () => string | null;
  onEvent: (event: ApiEvent) => void;
  onStatus: (status: ConnectionStatus) => void;
  /** Called after each (re)connect so the caller can refetch a snapshot. */
  onOpen?: () => void;
  fetchImpl?: typeof fetch;
  /** The server sends a heartbeat every 15 s. Silence for this long means a dead link. */
  silenceTimeoutMs?: number;
  backoffMs?: { initial: number; max: number };
}

/** Parse one stream line. Accepts bare JSON lines and SSE-style `data:` lines. Returns null for blanks and comments. */
export function parseStreamLine(line: string): ApiEvent | null {
  let text = line.trim();
  if (text === "" || text.startsWith(":")) return null;
  if (text.startsWith("data:")) text = text.slice(5).trim();
  try {
    const value: unknown = JSON.parse(text);
    if (value && typeof value === "object" && "type" in value) {
      const type = (value as { type: unknown }).type;
      if (type === "state" || type === "heartbeat") return value as ApiEvent;
    }
  } catch {
    // Ignore a malformed line rather than dropping the connection.
  }
  return null;
}

/**
 * Read `GET /api/events` with fetch streaming so the Bearer token travels in a
 * header. Reconnects with exponential backoff and jitter, treats a silent stream
 * as dead (heartbeat handling), and stops on 401. Returns a stop function.
 */
export function openEventStream(opts: EventStreamOptions): () => void {
  const doFetch = opts.fetchImpl ?? fetch.bind(globalThis);
  const silenceMs = opts.silenceTimeoutMs ?? 40_000;
  const backoff = opts.backoffMs ?? { initial: 1000, max: 15_000 };
  let stopped = false;
  let controller: AbortController | null = null;

  async function run() {
    let delay = backoff.initial;
    let first = true;
    while (!stopped) {
      opts.onStatus(first ? "connecting" : "reconnecting");
      first = false;
      controller = new AbortController();
      const current = controller;
      let watchdog: ReturnType<typeof setTimeout> | undefined;
      const arm = () => {
        clearTimeout(watchdog);
        watchdog = setTimeout(() => current.abort(), silenceMs);
      };
      try {
        const token = opts.getToken();
        arm();
        const res = await doFetch(opts.url, {
          headers: {
            Accept: "application/x-ndjson, text/event-stream",
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
          },
          signal: current.signal,
        });
        if (res.status === 401) {
          opts.onStatus("unauthorized");
          return;
        }
        if (!res.ok || !res.body) throw new Error(`stream failed: ${res.status}`);
        opts.onStatus("live");
        opts.onOpen?.();
        delay = backoff.initial;
        arm();
        const reader = res.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";
        for (;;) {
          const { done, value } = await reader.read();
          if (done) break;
          arm();
          buffer += decoder.decode(value, { stream: true });
          let nl = buffer.indexOf("\n");
          while (nl >= 0) {
            const event = parseStreamLine(buffer.slice(0, nl));
            buffer = buffer.slice(nl + 1);
            if (event) opts.onEvent(event);
            nl = buffer.indexOf("\n");
          }
        }
      } catch {
        // Fall through to reconnect, unless we were stopped.
      } finally {
        clearTimeout(watchdog);
      }
      if (stopped) return;
      opts.onStatus("reconnecting");
      await new Promise((r) => setTimeout(r, delay * (0.8 + Math.random() * 0.4)));
      delay = Math.min(delay * 2, backoff.max);
    }
  }

  void run();
  return () => {
    stopped = true;
    controller?.abort();
  };
}
