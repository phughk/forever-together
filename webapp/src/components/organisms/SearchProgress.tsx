import { SEARCH_LIMIT_S, STAGES, LAST_STAGE, WIDENING_STEP_S } from "../../api/constants";
import type { DungeonEntry, NetworkView, SearchView } from "../../api/types";
import { formatAge } from "../../lib/format";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { ProgressBar } from "../atoms/ProgressBar";
import { Text } from "../atoms/Typography";
import { Field } from "../molecules/Field";
import { DiagnosisSummary } from "../molecules/DiagnosisSummary";
import { StageStep } from "../molecules/StageStep";

export interface SearchProgressProps {
  search: SearchView;
  network: NetworkView;
  dungeons: DungeonEntry[];
  now: number;
  onStop: () => void | Promise<void>;
}

const BOOTSTRAP_COPY: Record<NetworkView["bootstrap"], string> = {
  remembered: "found through computers remembered from earlier runs",
  peer_exchange: "found through other players' lists of computers",
  trusted: "found through a trusted computer, the last resort",
  none: "no other computers found yet",
};

/** "3 other computers connected"; zero reads as not connected yet. */
function connectedPhrase(n: number): string {
  if (n <= 0) return "not connected to other computers yet";
  return `${n} other ${n === 1 ? "computer" : "computers"} connected`;
}

export function SearchProgress({ search, network, dungeons, now, onStop }: SearchProgressProps) {
  if (search.status === "idle") return null;

  const names = search.dungeons.map((s) => dungeons.find((d) => d.slug === s)?.name ?? s);
  const elapsedS = search.startedAt ? (now - search.startedAt) / 1000 : 0;
  const clockS = search.stageClockStartedAt ? (now - search.stageClockStartedAt) / 1000 : 0;
  const nextStageS = search.stage < LAST_STAGE ? (search.stage + 1) * WIDENING_STEP_S - clockS : null;
  const diagnosis = search.diagnosis ?? null;
  // Only an older server sends no diagnosis; then the old inline notice is the fallback.
  const lowCount = diagnosis === null && search.searching !== null && search.searching <= 3;

  if (search.status === "limit_reached") {
    return (
      <Card title="Search ended">
        {diagnosis ? (
          <>
            <Text>The search reached its {SEARCH_LIMIT_S / 60} minute limit without forming a party.</Text>
            <DiagnosisSummary diagnosis={diagnosis} elapsedS={SEARCH_LIMIT_S} dungeonCount={search.dungeons.length} />
          </>
        ) : (
          <Notice tone="warn">
            The search reached its {SEARCH_LIMIT_S / 60} minute limit without forming a party. Nobody is searching for you now.
            Start again, or try another dungeon from the list.
          </Notice>
        )}
        <Text size="small" muted>
          Searched: {names.join(", ")}
        </Text>
        <div className="ft-actions">
          <Button variant="ghost" onClick={() => void onStop()}>
            Clear
          </Button>
        </div>
      </Card>
    );
  }

  return (
    <Card title="Searching">
      <div>
        <Field label="Dungeons">{names.join(", ")}</Field>
        <Field label="Players seen">
          {diagnosis
            ? diagnosis.situation === "warming_up" || diagnosis.situation === "not_connected"
              ? "Not known yet"
              : `${diagnosis.sampleFull && diagnosis.seen.players > 0 ? "about " : ""}${diagnosis.seen.players} besides you (rough, from this computer)`
            : search.searching === null
              ? "Counting..."
              : `${search.searching} (rough, from this computer)`}
        </Field>
        <Field label="Network">
          {connectedPhrase(network.peers)}, {BOOTSTRAP_COPY[network.bootstrap]}
        </Field>
        <Field label="Searching for">{formatAge(elapsedS)}</Field>
      </div>

      <ProgressBar value={elapsedS / SEARCH_LIMIT_S} label="Time until the search limit" />
      <Text size="small" muted>
        {nextStageS !== null
          ? `Next widening in ${Math.ceil(Math.max(0, nextStageS))} s.`
          : "Already at the widest stage."}{" "}
        The search stops after {SEARCH_LIMIT_S / 60} minutes.
      </Text>

      {diagnosis && <DiagnosisSummary diagnosis={diagnosis} elapsedS={elapsedS} dungeonCount={search.dungeons.length} />}

      {search.stage === LAST_STAGE && lowCount && (
        <Notice tone="warn">
          Few players are searching this dungeon right now and the search is already as wide as it goes. Try other dungeons, or
          keep waiting.
        </Notice>
      )}

      <ol className="ft-list" aria-label="Search stages">
        {STAGES.map((s) => (
          <StageStep
            key={s.stage}
            number={s.stage}
            title={s.title}
            rule={s.rule}
            state={s.stage < search.stage ? "done" : s.stage === search.stage ? "current" : "upcoming"}
            startsInS={s.stage > search.stage ? Math.max(0, Math.ceil(s.startsAtS - clockS)) : undefined}
          />
        ))}
      </ol>

      <div className="ft-actions">
        <Button variant="danger" onClick={() => void onStop()}>
          Stop search
        </Button>
      </div>
    </Card>
  );
}
