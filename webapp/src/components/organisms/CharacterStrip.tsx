import { useState } from "react";
import { STALE_CODE_AFTER_S } from "../../api/constants";
import type { CharacterView } from "../../api/types";
import { formatAge } from "../../lib/format";
import "../../styles/finder.css";
import { Button } from "../atoms/Button";
import { CharacterSummary } from "../molecules/CharacterSummary";
import { CharacterPanel, type CharacterPanelProps } from "./CharacterPanel";

export interface CharacterStripProps extends Omit<CharacterPanelProps, "character" | "embedded" | "addonUrl"> {
  character: CharacterView;
  /** Start with the edit panel open (stories). */
  defaultOpen?: boolean;
}

/**
 * The idle screen's compact character line: the one-line summary with its badges, a short stale line,
 * and an "Edit" button that opens the full panel (details, update, remove) inline.
 */
export function CharacterStrip({ character, now, defaultOpen = false, ...panel }: CharacterStripProps) {
  const [open, setOpen] = useState(defaultOpen || !!panel.error);
  const ageS = (now - character.createdAt) / 1000;
  const stale = ageS > STALE_CODE_AFTER_S;
  return (
    <section className="ft-strip" aria-label="Your character">
      <div className="ft-strip__row">
        <CharacterSummary character={character} stale={stale} />
        <Button size="small" variant="ghost" aria-expanded={open} onClick={() => setOpen(!open)}>
          {open ? "Close" : "Edit"}
        </Button>
      </div>
      {stale && <p className="ft-strip__stale">Code is {formatAge(ageS)} old. Refresh in the addon.</p>}
      {open && <CharacterPanel {...panel} character={character} now={now} embedded />}
    </section>
  );
}
