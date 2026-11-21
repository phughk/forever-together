import type { DungeonEntry } from "../../api/types";
import "../../styles/finder.css";
import { Badge } from "../atoms/Badge";
import { Checkbox } from "../atoms/Toggle";

export interface DungeonOptionProps {
  dungeon: DungeonEntry;
  /** Combine mode: the row has a tick box and tapping it toggles. Default: the row is a button that starts the search. */
  multi?: boolean;
  /** Ticked. Only meaningful in combine mode. */
  selected?: boolean;
  /** Combine mode: the tick changed. */
  onToggle?: (slug: string, selected: boolean) => void;
  /** Single mode: the row was tapped, start the search for this dungeon. */
  onStart?: (slug: string) => void;
  /** The dungeon's zone equals the character's zone. */
  inZone?: boolean;
  disabled?: boolean;
}

function Meta({ dungeon, inZone }: { dungeon: DungeonEntry; inZone?: boolean }) {
  return (
    <span className="ft-dungeon__meta">
      <span>
        Levels {dungeon.levelMin} to {dungeon.levelMax}
      </span>
      <span>{dungeon.zone}</span>
      {inZone && <Badge tone="info">You are here</Badge>}
    </span>
  );
}

/** One dungeon row. In single mode the whole row is the action; in combine mode it is a tick box. */
export function DungeonOption({ dungeon, multi, selected = false, onToggle, onStart, inZone, disabled }: DungeonOptionProps) {
  if (!multi) {
    return (
      <button
        type="button"
        className={["ft-dungeon", disabled && "ft-dungeon--disabled"].filter(Boolean).join(" ")}
        disabled={disabled}
        aria-label={`Search ${dungeon.name}`}
        onClick={() => onStart?.(dungeon.slug)}
      >
        <span className="ft-dungeon__body">
          <span className="ft-dungeon__name">{dungeon.name}</span>
          <Meta dungeon={dungeon} inZone={inZone} />
        </span>
        <span className="ft-dungeon__go" aria-hidden="true">
          ›
        </span>
      </button>
    );
  }
  return (
    <label className={["ft-dungeon", selected && "ft-dungeon--selected", disabled && "ft-dungeon--disabled"].filter(Boolean).join(" ")}>
      <Checkbox
        checked={selected}
        disabled={disabled}
        aria-label={`Include ${dungeon.name}`}
        onChange={(checked) => onToggle?.(dungeon.slug, checked)}
      />
      <span className="ft-dungeon__body">
        <span className="ft-dungeon__name">{dungeon.name}</span>
        <Meta dungeon={dungeon} inZone={inZone} />
      </span>
    </label>
  );
}
