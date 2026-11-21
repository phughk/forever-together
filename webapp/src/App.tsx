import { useCallback, useEffect, useState } from "react";
import { installAddonFlow, type AddonResult } from "./api/addonFlow";
import { describeAddonError, describeCodeError, isApiError, isLoopbackOnly } from "./api/errors";
import type { AddonInfo, Api, DungeonsResponse, Settings } from "./api/types";
import { ApiProvider, useAppState } from "./api/useAppState";
import { NotPairedPage } from "./components/pages/NotPairedPage";
import { FinderPage } from "./components/pages/FinderPage";
import { SettingsPage } from "./components/pages/SettingsPage";
import type { PairingState } from "./components/organisms/LanSettings";
import type { View } from "./components/organisms/AppHeader";

const LOOPBACK = new Set(["localhost", "127.0.0.1", "[::1]", "::1"]);
const EMPTY_DUNGEONS: DungeonsResponse = { dungeons: [], recommended: [] };

function failureText(e: unknown, fallback: string): string {
  if (isLoopbackOnly(e)) return "That can only be changed in the desktop app.";
  return isApiError(e) && e.message ? e.message : fallback;
}

/** Connects the pages to the API: state stream, dungeon list, form state and settings calls. */
function Container({ api }: { api: Api }) {
  const { state, status, now } = useAppState(api);
  const [view, setView] = useState<View>("finder");
  const [dungeons, setDungeons] = useState<DungeonsResponse>(EMPTY_DUNGEONS);
  const [codeError, setCodeError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [dismissedParty, setDismissedParty] = useState<string | null>(null);
  const [remote, setRemote] = useState(!LOOPBACK.has(window.location.hostname));
  const [pairing, setPairing] = useState<PairingState>({ kind: "off" });
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const [addon, setAddon] = useState<AddonInfo | null>(null);
  const [addonResult, setAddonResult] = useState<AddonResult | null>(null);
  const [addonBusy, setAddonBusy] = useState(false);
  const [addonFolderError, setAddonFolderError] = useState<string | null>(null);

  const characterKey = state?.character ? `${state.character.level}|${state.character.zone}|${state.character.createdAt}` : "none";
  const hasState = state !== null;

  // The dungeon list and recommendations depend on the character, so reload when it changes.
  useEffect(() => {
    if (!hasState) return;
    let cancelled = false;
    api
      .getDungeons()
      .then((d) => !cancelled && setDungeons(d))
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [api, hasState, characterKey]);

  const lanMode = state?.settings.lan_mode ?? false;
  const lanAddress = state?.settings.lan_address ?? undefined;
  useEffect(() => {
    if (view !== "settings" || !lanMode) {
      setPairing({ kind: "off" });
      return;
    }
    let cancelled = false;
    setPairing({ kind: "loading" });
    api
      .getPairing(lanAddress)
      .then((p) => !cancelled && setPairing({ kind: "ready", url: p.url, address: p.address, addresses: p.addresses }))
      .catch((e: unknown) => {
        if (cancelled) return;
        if (isLoopbackOnly(e)) {
          setRemote(true);
          setPairing({ kind: "forbidden" });
        } else {
          setPairing({ kind: "error", message: failureText(e, "Could not get the phone link.") });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [api, view, lanMode, lanAddress]);

  // The addon card reads the installer state each time Settings opens. A phone gets 403 and the card locks.
  useEffect(() => {
    if (view !== "settings") return;
    let cancelled = false;
    setAddonResult(null);
    setAddonFolderError(null);
    api
      .getAddon()
      .then((a) => !cancelled && setAddon(a))
      .catch((e: unknown) => {
        if (!cancelled && isLoopbackOnly(e)) setRemote(true);
      });
    return () => {
      cancelled = true;
    };
  }, [api, view]);

  const installAddon = useCallback(async () => {
    if (!addon) return;
    setAddonBusy(true);
    setAddonResult(null);
    setAddonFolderError(null);
    const out = await installAddonFlow(api, addon);
    if (out.ok) setAddon(out.info);
    else if (out.remote) setRemote(true);
    setAddonResult(out.result);
    setAddonBusy(false);
  }, [api, addon]);

  const saveAddonFolder = useCallback(
    async (folder: string) => {
      setAddonBusy(true);
      setAddonResult(null);
      setAddonFolderError(null);
      try {
        setAddon(await api.setAddonFolder(folder));
        return true;
      } catch (e) {
        if (isLoopbackOnly(e)) setRemote(true);
        setAddonFolderError(isApiError(e) && e.message ? e.message : describeAddonError(isApiError(e) ? e.code : ""));
        return false;
      } finally {
        setAddonBusy(false);
      }
    },
    [api],
  );

  const changeSettings = useCallback(
    async (patch: Partial<Settings>) => {
      setSettingsError(null);
      try {
        await api.putSettings(patch);
      } catch (e) {
        if (isLoopbackOnly(e)) setRemote(true);
        setSettingsError(failureText(e, "The setting could not be saved."));
      }
    },
    [api],
  );

  if (status === "unauthorized") return <NotPairedPage />;
  if (!state) return null;

  const shown = state.party && state.party.phase === "dropped" && state.party.id === dismissedParty ? { ...state, party: null } : state;

  if (view === "settings") {
    return (
      <SettingsPage
        settings={state.settings}
        status={status}
        pairing={pairing}
        remote={remote}
        error={settingsError}
        onNavigate={setView}
        addon={addon}
        addonResult={addonResult}
        addonBusy={addonBusy}
        addonFolderError={addonFolderError}
        onInstallAddon={installAddon}
        onSaveAddonFolder={saveAddonFolder}
        onLanModeChange={(on) => changeSettings({ lan_mode: on })}
        onLanAddressChange={(a) => changeSettings({ lan_address: a })}
        onResetToken={async () => {
          try {
            await api.resetToken();
          } catch (e) {
            setSettingsError(failureText(e, "Phones could not be signed out."));
          }
        }}
        onClearHistory={async () => {
          try {
            await api.clearHistory();
          } catch (e) {
            if (isLoopbackOnly(e)) setRemote(true);
            setSettingsError(failureText(e, "The search history could not be cleared."));
          }
        }}
        onKnownPeersSizeChange={(n) => changeSettings({ known_peers_size: n })}
        onAddTrusted={(k) =>
          changeSettings({ trusted_peers: [...state.settings.trusted_peers.filter((x) => x !== k), k] })
        }
        onRemoveTrusted={(k) => changeSettings({ trusted_peers: state.settings.trusted_peers.filter((x) => x !== k) })}
      />
    );
  }

  return (
    <FinderPage
      state={shown}
      dungeons={dungeons}
      now={now}
      status={status}
      codeError={codeError}
      busy={busy}
      onNavigate={setView}
      onSubmitCode={async (code) => {
        setBusy(true);
        setCodeError(null);
        try {
          await api.submitCharacter(code);
        } catch (e) {
          setCodeError(describeCodeError(e));
        } finally {
          setBusy(false);
        }
      }}
      onForgetCharacter={() => api.forgetCharacter()}
      onStartSearch={async (slugs) => {
        setBusy(true);
        try {
          await api.startSearch(slugs);
        } finally {
          setBusy(false);
        }
      }}
      onStopSearch={() => api.stopSearch()}
      onConfirmParty={() => api.confirmParty()}
      onDeclineParty={() => api.declineParty()}
      onDismissParty={() => setDismissedParty(state.party?.id ?? null)}
    />
  );
}

export function App({ api }: { api: Api }) {
  return (
    <ApiProvider api={api}>
      <Container api={api} />
    </ApiProvider>
  );
}
