import { useEffect, useState, type KeyboardEvent } from "react";
import { STALE_CODE_AFTER_S } from "../../api/constants";
import type { CharacterView } from "../../api/types";
import { formatAge } from "../../lib/format";
import "../../styles/finder.css";
import { Button } from "../atoms/Button";
import { CharacterSummary } from "../molecules/CharacterSummary";
import { CodePasteForm } from "../molecules/CodePasteForm";
import { CharacterPanel, type CharacterPanelProps } from "./CharacterPanel";

export interface CharacterStripProps extends Omit<CharacterPanelProps, "character" | "embedded" | "addonUrl"> {
  character: CharacterView;
  /** Start with the paste form open (stories). */
  defaultOpen?: boolean;
  /** A search or party is running: Change character is disabled and this short reason is shown. */
  lockedReason?: string;
}

function SwapIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false" className="ft-button__icon">
      <path d="M4 8h13M13 4l4 4-4 4M20 16H7M11 12l-4 4 4 4" />
    </svg>
  );
}

/**
 * The compact character block shown beside every finder screen: the one-line summary with its badges and one
 * obvious "Change character" button that opens the paste form inline. When the code is out of date the same
 * button reads "Update" and is the primary action. Remove lives behind Details, as a secondary action.
 */
export function CharacterStrip({ character, now, defaultOpen = false, lockedReason, ...panel }: CharacterStripProps) {
  const [open, setOpen] = useState(defaultOpen || !!panel.error);
  useEffect(() => {
    if (panel.error) setOpen(true);
  }, [panel.error]);
  const ageS = (now - character.createdAt) / 1000;
  const stale = ageS > STALE_CODE_AFTER_S;
  const locked = !!lockedReason;
  const showForm = open && !locked;
  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") setOpen(false);
  };
  return (
    <section className="ft-strip" aria-label="Your character" onKeyDown={onKeyDown}>
      <div className="ft-strip__row">
        <CharacterSummary character={character} stale={stale} />
        <Button
          className="ft-strip__change"
          variant={stale ? "primary" : "default"}
          aria-expanded={showForm}
          disabled={locked}
          onClick={() => setOpen(!open)}
        >
          <SwapIcon />
          {stale ? "Update" : "Change character"}
        </Button>
      </div>
      {locked && <p className="ft-strip__note">{lockedReason}</p>}
      {stale && !showForm && !locked && <p className="ft-strip__stale">Code is {formatAge(ageS)} old. Paste a new one.</p>}
      {showForm && (
        <div className="ft-strip__form">
          <CodePasteForm
            autoFocus
            error={panel.error}
            busy={panel.busy}
            onCancel={() => setOpen(false)}
            onSubmit={async (c) => {
              await panel.onSubmitCode(c);
              setOpen(false);
            }}
          />
        </div>
      )}
      {!locked && <CharacterPanel {...panel} character={character} now={now} embedded />}
    </section>
  );
}
