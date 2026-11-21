import { STALE_CODE_AFTER_S } from "../../api/constants";
import type { AppState, ConnectionStatus, DungeonsResponse } from "../../api/types";
import { finderScreen } from "../../lib/screen";
import { Notice } from "../atoms/Notice";
import { CharacterSummary } from "../molecules/CharacterSummary";
import { AppHeader, type View } from "../organisms/AppHeader";
import { CharacterPanel } from "../organisms/CharacterPanel";
import { DungeonPicker } from "../organisms/DungeonPicker";
import { PartyPanel } from "../organisms/PartyPanel";
import { SearchProgress } from "../organisms/SearchProgress";
import { FinderTemplate } from "../templates/FinderTemplate";

export interface FinderActions {
  onNavigate: (view: View) => void;
  onSubmitCode: (code: string) => void | Promise<void>;
  onForgetCharacter: () => void | Promise<void>;
  /** Starts a search for these dungeons (a tap on a dungeon, "Search N", or "Search again"). */
  onStartSearch: (slugs: string[]) => void | Promise<void>;
  /** Stops a running search, or clears an ended one. */
  onStopSearch: () => void | Promise<void>;
  onConfirmParty: () => void | Promise<void>;
  onDeclineParty: () => void | Promise<void>;
  onDismissParty?: () => void;
}

export interface FinderPageProps extends FinderActions {
  state: AppState;
  dungeons: DungeonsResponse;
  now: number;
  status: ConnectionStatus;
  /** Plain-language reason the last pasted code was refused. */
  codeError?: string | null;
  busy?: boolean;
}

/** The finder with real content. One screen per state (see `finderScreen`), plus a one-line strip about the character. */
export function FinderPage({ state, dungeons, now, status, codeError, busy, ...actions }: FinderPageProps) {
  const { character, search, party } = state;
  const screen = finderScreen(state);

  const characterPanel = (
    <CharacterPanel
      character={character}
      now={now}
      onSubmitCode={actions.onSubmitCode}
      onForget={actions.onForgetCharacter}
      busy={busy}
      error={codeError}
    />
  );
  const picker = (
    <DungeonPicker
      dungeons={dungeons.dungeons}
      recommended={dungeons.recommended}
      onStart={actions.onStartSearch}
      zone={character?.zone}
      needsCharacter={!character}
      busy={busy}
    />
  );

  let main;
  switch (screen) {
    case "first_run":
      main = (
        <>
          {characterPanel}
          {picker}
        </>
      );
      break;
    case "idle":
      main = picker;
      break;
    case "searching":
    case "ended":
      main = (
        <SearchProgress
          search={search}
          dungeons={dungeons.dungeons}
          now={now}
          onStop={actions.onStopSearch}
          onSearchAgain={() => void actions.onStartSearch(search.dungeons)}
          onChangeDungeon={() => void actions.onStopSearch()}
        />
      );
      break;
    default:
      main = (
        <PartyPanel
          party={party!}
          now={now}
          onConfirm={actions.onConfirmParty}
          onDecline={actions.onDeclineParty}
          onDismiss={actions.onDismissParty}
        />
      );
  }

  // The strip: idle shows the character card (update and remove live there); the other screens only a one-line reminder.
  const strip =
    screen === "idle" ? (
      characterPanel
    ) : character && screen !== "first_run" ? (
      <CharacterSummary character={character} stale={(now - character.createdAt) / 1000 > STALE_CODE_AFTER_S} />
    ) : undefined;

  return (
    <FinderTemplate
      header={<AppHeader view="finder" onNavigate={actions.onNavigate} status={status} />}
      notices={
        status === "reconnecting" ? (
          <Notice tone="warn">The connection to the app was lost. Showing the last known state while reconnecting.</Notice>
        ) : undefined
      }
      strip={strip}
      main={main}
    />
  );
}
