import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import type { Api, AppState, ConnectionStatus } from "./types";

const ApiContext = createContext<Api | null>(null);

export function ApiProvider({ api, children }: { api: Api; children: ReactNode }) {
  return <ApiContext.Provider value={api}>{children}</ApiContext.Provider>;
}

export function useApi(): Api {
  const api = useContext(ApiContext);
  if (!api) throw new Error("useApi needs an <ApiProvider>");
  return api;
}

export interface AppStateHook {
  state: AppState | null;
  status: ConnectionStatus;
  /** Server-clock "now" in ms, ticking once a second and corrected for clock difference. */
  now: number;
}

/** Subscribes to the event stream and returns the latest snapshot, connection status and a server-corrected clock. */
export function useAppState(api: Api): AppStateHook {
  const [state, setState] = useState<AppState | null>(null);
  const [status, setStatus] = useState<ConnectionStatus>("connecting");
  const [now, setNow] = useState(0);
  // Server time minus local time at the moment a snapshot arrived.
  const offset = useRef(0);

  useEffect(() => {
    return api.subscribe({
      onState: (next) => {
        offset.current = next.serverNow - Date.now();
        setNow(next.serverNow);
        setState(next);
      },
      onStatus: setStatus,
    });
  }, [api]);

  useEffect(() => {
    const id = setInterval(() => setNow(Date.now() + offset.current), 1000);
    return () => clearInterval(id);
  }, []);

  return { state, status, now };
}
