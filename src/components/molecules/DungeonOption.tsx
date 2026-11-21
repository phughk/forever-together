import type { DungeonEntry } from "../../api/types";
import { Badge } from "../atoms/Badge";
import { Checkbox } from "../atoms/Toggle";

export interface DungeonOptionProps {
  dungeon: DungeonEntry;
  selected: boolean;
  onToggle: (slug: string, selected: boolean) => void;
  /** The dungeon's zone equals the character's zone. */
  inZone?: boolean;
  disabled?: boolean;
}

export function DungeonOption({ dungeon, selected, onToggle, inZone, disabled }: DungeonOptionProps) {
  const comingSoon = dungeon.status === "coming_soon";
  const off = disabled || comingSoon;
  return (
    <label className={["ft-dungeon", selected && "ft-dungeon--selected", off && "ft-dungeon--disabled"].filter(Boolean).join(" ")}>
      <Checkbox
        checked={selected}
        disabled={off}
        aria-label={`Search ${dungeon.name}`}
        onChange={(checked) => onToggle(dungeon.slug, checked)}
      />
      <span className="ft-dungeon__body">
        <span className="ft-dungeon__name">{dungeon.name}</span>
        <span className="ft-dungeon__meta">
          <span>
            Levels {dungeon.levelMin} to {dungeon.levelMax}
          </span>
          <span>{dungeon.zone}</span>
          {inZone && <Badge tone="info">You are here</Badge>}
          {comingSoon && <Badge tone="warn">Coming soon</Badge>}
        </span>
      </span>
    </label>
  );
}
