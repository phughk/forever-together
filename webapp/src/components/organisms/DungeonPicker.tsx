import type { DungeonEntry } from "../../api/types";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { DungeonOption } from "../molecules/DungeonOption";

export interface DungeonPickerProps {
  dungeons: DungeonEntry[];
  /** Slugs in recommendation order. May be empty. */
  recommended: string[];
  selected: string[];
  onSelectionChange: (slugs: string[]) => void;
  onStart: (slugs: string[]) => void | Promise<void>;
  /** The character's zone, to flag the dungeon you are standing near. */
  zone?: string;
  /** No character yet. */
  needsCharacter?: boolean;
  /** A search is running, so the choice is frozen. */
  searching?: boolean;
  busy?: boolean;
}

export function DungeonPicker({
  dungeons,
  recommended,
  selected,
  onSelectionChange,
  onStart,
  zone,
  needsCharacter,
  searching,
  busy,
}: DungeonPickerProps) {
  const bySlug = new Map(dungeons.map((d) => [d.slug, d]));
  const recommendedEntries = recommended.flatMap((s) => bySlug.get(s) ?? []);
  const recSet = new Set(recommended);
  const others = dungeons.filter((d) => d.status === "available" && !recSet.has(d.slug));
  const comingSoon = dungeons.filter((d) => d.status === "coming_soon");

  const toggle = (slug: string, on: boolean) =>
    onSelectionChange(on ? [...selected.filter((s) => s !== slug), slug] : selected.filter((s) => s !== slug));
  const row = (d: DungeonEntry) => (
    <li key={d.slug}>
      <DungeonOption
        dungeon={d}
        selected={selected.includes(d.slug)}
        onToggle={toggle}
        inZone={!!zone && d.zone.toLowerCase() === zone.toLowerCase()}
        disabled={searching || needsCharacter}
      />
    </li>
  );

  return (
    <Card title="Dungeons">
      {needsCharacter ? (
        <Notice tone="info">Paste your character code first. The app recommends dungeons from your level and zone.</Notice>
      ) : (
        <>
          <Text size="small" muted>
            Pick one dungeon, or several to search them together. Your location is only a hint, nothing is refused.
          </Text>
          {recommendedEntries.length === 0 && (
            <Notice tone="warn">
              No dungeon is recommended for this level. You can still pick any available dungeon below.
            </Notice>
          )}
        </>
      )}

      {recommendedEntries.length > 0 && (
        <>
          <h3 className="ft-section-label">Recommended for you</h3>
          <ul className="ft-list">{recommendedEntries.map(row)}</ul>
        </>
      )}
      {others.length > 0 && (
        <>
          <h3 className="ft-section-label">{recommendedEntries.length > 0 ? "Other dungeons" : "Available dungeons"}</h3>
          <ul className="ft-list">{others.map(row)}</ul>
        </>
      )}
      {comingSoon.length > 0 && (
        <>
          <h3 className="ft-section-label">Coming soon</h3>
          <ul className="ft-list">{comingSoon.map(row)}</ul>
        </>
      )}

      <div className="ft-actions">
        <Button
          variant="primary"
          disabled={needsCharacter || searching || busy || selected.length === 0}
          onClick={() => void onStart(selected)}
        >
          {selected.length > 1 ? `Search ${selected.length} dungeons` : "Start search"}
        </Button>
        <Button
          disabled={needsCharacter || searching || recommendedEntries.length === 0}
          onClick={() => onSelectionChange(recommendedEntries.filter((d) => d.status === "available").map((d) => d.slug))}
        >
          Select all recommended
        </Button>
      </div>
    </Card>
  );
}
