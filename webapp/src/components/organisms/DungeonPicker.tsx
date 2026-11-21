import { useState } from "react";
import type { DungeonEntry } from "../../api/types";
import "../../styles/finder.css";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Switch } from "../atoms/Toggle";
import { Heading } from "../atoms/Typography";
import { DungeonOption } from "../molecules/DungeonOption";
import { FirstRunStep } from "../molecules/FirstRunStep";

export interface DungeonPickerProps {
  dungeons: DungeonEntry[];
  /** Slugs in recommendation order. May be empty. */
  recommended: string[];
  /** Tapping a dungeon (or "Search N" in combine mode) starts the search for these slugs. */
  onStart: (slugs: string[]) => void | Promise<void>;
  /** The character's zone, to flag the dungeon you are standing near. */
  zone?: string;
  /** No character yet: only the greyed "Pick a dungeon" step is shown, no list. */
  needsCharacter?: boolean;
  /** A search is running, so the choice is frozen. */
  searching?: boolean;
  /** A request is in flight. Rows are disabled. */
  busy?: boolean;
  /** Start with "Combine dungeons" on. */
  defaultCombine?: boolean;
}

export function DungeonPicker({
  dungeons,
  recommended,
  onStart,
  zone,
  needsCharacter,
  searching,
  busy,
  defaultCombine = false,
}: DungeonPickerProps) {
  const [combine, setCombine] = useState(defaultCombine);
  const [selected, setSelected] = useState<string[]>([]);

  if (needsCharacter) {
    return (
      <Card title="Dungeons">
        <ol className="ft-firststeps">
          <FirstRunStep n={4} title="Pick a dungeon" pending />
        </ol>
      </Card>
    );
  }

  const bySlug = new Map(dungeons.map((d) => [d.slug, d]));
  const recommendedEntries = recommended.flatMap((s) => bySlug.get(s) ?? []);
  const recSet = new Set(recommended);
  const others = dungeons.filter((d) => !recSet.has(d.slug));
  const frozen = !!searching || !!busy;

  const toggle = (slug: string, on: boolean) =>
    setSelected(on ? [...selected.filter((s) => s !== slug), slug] : selected.filter((s) => s !== slug));
  const row = (d: DungeonEntry) => (
    <li key={d.slug}>
      <DungeonOption
        dungeon={d}
        multi={combine}
        selected={selected.includes(d.slug)}
        onToggle={toggle}
        onStart={(slug) => void onStart([slug])}
        inZone={!!zone && d.zone.toLowerCase() === zone.toLowerCase()}
        disabled={frozen}
      />
    </li>
  );

  return (
    <Card className="ft-picker">
      <div className="ft-picker__head">
        <Heading level={2} className="ft-card__title">
          Dungeons
        </Heading>
        <Switch
          checked={combine}
          disabled={frozen}
          onChange={(on) => {
            setCombine(on);
            if (!on) setSelected([]);
          }}
        >
          Combine dungeons
        </Switch>
      </div>
      {recommendedEntries.length === 0 && <Notice tone="warn">No dungeon is recommended for this level.</Notice>}

      <div className="ft-picker__region">
        {recommendedEntries.length > 0 && (
          <>
            <h3 className="ft-section-label">Recommended for you</h3>
            <ul className="ft-list">{recommendedEntries.map(row)}</ul>
          </>
        )}
        {others.length > 0 && (
          <>
            <h3 className="ft-section-label">{recommendedEntries.length > 0 ? "Other dungeons" : "All dungeons"}</h3>
            <ul className="ft-list">{others.map(row)}</ul>
          </>
        )}
        {combine && (
          <div className="ft-picker__bar">
            <span>{selected.length} selected</span>
            <Button variant="primary" disabled={frozen || selected.length === 0} onClick={() => void onStart(selected)}>
              Search {selected.length}
            </Button>
          </div>
        )}
      </div>
    </Card>
  );
}
