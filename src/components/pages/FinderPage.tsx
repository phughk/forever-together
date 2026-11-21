import type { AppState, ConnectionStatus, DungeonsResponse } from "../../api/types";
import { Notice } from "../atoms/Notice";
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
  onSelectionChange: (slugs: string[]) => void;
  onStartSearch: (slugs: string[]) => void | Promise<void>;
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
  selected: string[];
  /** Plain-language reason the last pasted code was refused. */
  codeError?: string | null;
  busy?: boolean;
}

/** The finder screen with real content: character, dungeons, live search and party. */
export function FinderPage({ state, dungeons, now, status, selected, codeError, busy, ...actions }: FinderPageProps) {
  const { character, search, party } = state;
  const searching = search.status === "searching";
  const partyActive = party !== null;

  const top = partyActive ? (
    <PartyPanel
      party={party}
      now={now}
      onConfirm={actions.onConfirmParty}
      onDecline={actions.onDeclineParty}
      onDismiss={actions.onDismissParty}
    />
  ) : search.status !== "idle" ? (
    <SearchProgress search={search} network={state.network} dungeons={dungeons.dungeons} now={now} onStop={actions.onStopSearch} />
  ) : undefined;

  return (
    <FinderTemplate
      header={<AppHeader view="finder" onNavigate={actions.onNavigate} status={status} />}
      notices={
        status === "reconnecting" ? (
          <Notice tone="warn">The connection to the app was lost. Showing the last known state while reconnecting.</Notice>
        ) : undefined
      }
      top={top}
      left={
        <CharacterPanel
          character={character}
          now={now}
          onSubmitCode={actions.onSubmitCode}
          onForget={actions.onForgetCharacter}
          busy={busy}
          error={codeError}
          locked={searching || partyActive}
        />
      }
      right={
        <DungeonPicker
          dungeons={dungeons.dungeons}
          recommended={dungeons.recommended}
          selected={selected}
          onSelectionChange={actions.onSelectionChange}
          onStart={actions.onStartSearch}
          zone={character?.zone}
          needsCharacter={!character}
          searching={searching || partyActive}
          busy={busy}
        />
      }
    />
  );
}
