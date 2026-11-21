import { SEARCH_LIMIT_S } from "../../api/constants";
import type { DungeonEntry, NetworkView, SearchView } from "../../api/types";
import { formatAge } from "../../lib/format";
import { Badge } from "../atoms/Badge";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { ProgressBar } from "../atoms/ProgressBar";
import { Heading, Text } from "../atoms/Typography";
import { DiagnosisSummary } from "../molecules/DiagnosisSummary";
import { SearchingWiderChip } from "../molecules/SearchingWiderChip";
import "../../styles/search.css";

export interface SearchProgressProps {
  search: SearchView;
  /** No longer shown (the header's status dot covers the connection). Still accepted so callers keep compiling. */
  network?: NetworkView;
  dungeons: DungeonEntry[];
  now: number;
  onStop: () => void | Promise<void>;
  /** Ended state: start the same search again. */
  onSearchAgain?: () => void;
  /** Ended state: go back to the dungeon list. */
  onChangeDungeon?: () => void;
}

/**
 * The search card. Running: Stop at the top, one bar for time to the limit, a "Searching wider" chip once the
 * search has widened, and what is seen from here. Ended: the result and what to do next. Without the two
 * optional callbacks the ended card has a single "OK" that calls `onStop`.
 */
export function SearchProgress({ search, dungeons, now, onStop, onSearchAgain, onChangeDungeon }: SearchProgressProps) {
  if (search.status === "idle") return null;

  const names = search.dungeons.map((s) => dungeons.find((d) => d.slug === s)?.name ?? s).join(", ");
  const elapsedS = search.startedAt ? (now - search.startedAt) / 1000 : 0;
  const diagnosis = search.diagnosis ?? null;

  if (search.status === "limit_reached") {
    const hasNext = Boolean(onSearchAgain || onChangeDungeon);
    return (
      <Card title="Search ended">
        <Text size="small" muted>
          {names}
        </Text>
        {diagnosis ? (
          <DiagnosisSummary diagnosis={diagnosis} elapsedS={SEARCH_LIMIT_S} dungeonCount={search.dungeons.length} />
        ) : (
          <Notice tone="warn">No party formed.</Notice>
        )}
        <div className="ft-actions">
          {onSearchAgain && (
            <Button variant="primary" onClick={onSearchAgain}>
              Search again
            </Button>
          )}
          {onChangeDungeon && <Button onClick={onChangeDungeon}>Change dungeon</Button>}
          {!hasNext && (
            <Button variant="primary" onClick={() => void onStop()}>
              OK
            </Button>
          )}
        </div>
      </Card>
    );
  }

  return (
    <Card>
      <div className="ft-search__top">
        <Heading level={2}>Searching</Heading>
        <Button variant="danger" onClick={() => void onStop()}>
          Stop search
        </Button>
      </div>
      <Text size="small" muted>
        {names}
      </Text>
      <div>
        <ProgressBar value={elapsedS / SEARCH_LIMIT_S} label="Time until the search limit" />
        <div className="ft-search__time">
          <Text size="small" muted>
            {formatAge(elapsedS)}
          </Text>
          <Text size="small" muted>
            of {SEARCH_LIMIT_S / 60} min
          </Text>
        </div>
      </div>
      {search.stage > 0 && <SearchingWiderChip />}
      {diagnosis ? (
        <DiagnosisSummary diagnosis={diagnosis} elapsedS={elapsedS} dungeonCount={search.dungeons.length} />
      ) : (
        // An older server sends no diagnosis: only the rough count of players is known.
        <Badge>{search.searching === null ? "Counting players..." : `~${search.searching} other players seen`}</Badge>
      )}
    </Card>
  );
}
