import { SEARCH_LIMIT_S } from "../../api/constants";
import type { DungeonEntry, SearchView } from "../../api/types";
import { Badge } from "../atoms/Badge";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Entity, Heading, Text } from "../atoms/Typography";
import { DiagnosisSummary } from "../molecules/DiagnosisSummary";
import { SearchClock } from "../molecules/SearchClock";
import { SearchingWiderChip } from "../molecules/SearchingWiderChip";
import "../../styles/search.css";

export interface SearchProgressProps {
  search: SearchView;
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

  const names = search.dungeons.map((s, i) => (
    <span key={s}>
      {i > 0 && ", "}
      <Entity>{dungeons.find((d) => d.slug === s)?.name ?? s}</Entity>
    </span>
  ));
  const elapsedS = search.startedAt ? (now - search.startedAt) / 1000 : 0;
  const diagnosis = search.diagnosis ?? null;

  const seen = diagnosis ? (
    <Card className="ft-search__seen">
      <DiagnosisSummary diagnosis={diagnosis} elapsedS={search.status === "limit_reached" ? SEARCH_LIMIT_S : elapsedS} dungeonCount={search.dungeons.length} />
    </Card>
  ) : null;

  if (search.status === "limit_reached") {
    const hasNext = Boolean(onSearchAgain || onChangeDungeon);
    return (
      <>
        <Card title="Search ended" className="ft-search__main">
          <Text className="ft-search__names">{names}</Text>
          {!diagnosis && <Notice tone="warn">No party formed.</Notice>}
          <div className="ft-actions ft-search__actions">
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
        {seen}
      </>
    );
  }

  return (
    <>
      <Card className="ft-search__main">
        <div className="ft-search__top">
          <Heading level={2}>Searching</Heading>
          <Button variant="danger" onClick={() => void onStop()}>
            Stop search
          </Button>
        </div>
        <Text className="ft-search__names">{names}</Text>
        <SearchClock elapsedS={elapsedS} limitS={SEARCH_LIMIT_S} />
        {search.stage > 0 && <SearchingWiderChip />}
      </Card>
      {seen ?? (
        // An older server sends no diagnosis: only the rough count of players is known.
        <Card>
          <Badge>{search.searching === null ? "Counting players..." : `~${search.searching} other players seen`}</Badge>
        </Card>
      )}
    </>
  );
}
