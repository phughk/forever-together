import { useState } from "react";
import { STALE_CODE_AFTER_S } from "../../api/constants";
import type { CharacterView } from "../../api/types";
import { formatAge } from "../../lib/format";
import { ADDON_URL } from "../../lib/links";
import "../../styles/finder.css";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { CopyChip } from "../atoms/CopyChip";
import { Notice } from "../atoms/Notice";
import { Entity } from "../atoms/Typography";
import { CharacterSummary } from "../molecules/CharacterSummary";
import { CodePasteForm } from "../molecules/CodePasteForm";
import { Details } from "../molecules/Details";
import { Field } from "../molecules/Field";
import { FirstRunStep } from "../molecules/FirstRunStep";

export interface CharacterPanelProps {
  character: CharacterView | null;
  /** Server-clock now in ms. */
  now: number;
  onSubmitCode: (code: string) => void | Promise<void>;
  onForget: () => void | Promise<void>;
  busy?: boolean;
  /** Plain-language reason the last pasted code was refused. */
  error?: string | null;
  /** Where "Get the addon" points. Defaults to ADDON_URL; null renders plain text. */
  addonUrl?: string | null;
  /** Only the secondary part: details and Remove, without the card, the summary or the update form (the strip shows those). */
  embedded?: boolean;
}

export function CharacterPanel({ character, now, onSubmitCode, onForget, busy, error, addonUrl = ADDON_URL, embedded }: CharacterPanelProps) {
  const [replacing, setReplacing] = useState(false);

  if (!character) {
    // The app cannot know whether the addon is installed, so it never says so: it just asks for the code.
    return (
      <Card title="Your character">
        <ol className="ft-firststeps">
          <FirstRunStep
            n={1}
            title={addonUrl ? <a href={addonUrl} target="_blank" rel="noreferrer">Get the addon</a> : "Get the addon"}
          />
          <FirstRunStep
            n={2}
            title={
              <>
                Type <CopyChip text="/wowft" />, copy
              </>
            }
          />
          <FirstRunStep n={3} title="Paste here">
            <CodePasteForm onSubmit={onSubmitCode} busy={busy} error={error} />
          </FirstRunStep>
        </ol>
      </Card>
    );
  }

  const ageS = (now - character.createdAt) / 1000;
  const stale = ageS > STALE_CODE_AFTER_S;
  const showForm = !embedded && (replacing || !!error);

  const body = (
    <>
      {!embedded && <CharacterSummary character={character} stale={stale} />}
      {!embedded && stale && <Notice tone="warn">Code is {formatAge(ageS)} old. Refresh in the addon.</Notice>}
      <Details summary="Details">
        <Field label="Realm"><Entity>{character.realmName}</Entity></Field>
        <Field label="Region"><Entity>{character.regionName}</Entity></Field>
        <Field label="Zone">{character.zone ? <Entity>{character.zone}</Entity> : "Unknown"}</Field>
        <Field label="Updated">{formatAge(ageS)} ago</Field>
        {embedded && (
          <div className="ft-actions">
            <Button size="small" variant="ghost" onClick={() => void onForget()}>
              Remove character
            </Button>
          </div>
        )}
      </Details>
      {showForm && (
        <CodePasteForm
          onSubmit={async (c) => {
            await onSubmitCode(c);
            setReplacing(false);
          }}
          busy={busy}
          error={error}
        />
      )}
      {!embedded && (
      <div className="ft-actions">
        {!showForm && (
          <Button size="small" variant={stale ? "primary" : "default"} onClick={() => setReplacing(true)}>
            {stale ? "Update" : "Change character"}
          </Button>
        )}
        {showForm && !error && (
          <Button size="small" variant="ghost" onClick={() => setReplacing(false)}>
            Cancel
          </Button>
        )}
        <Button size="small" variant="ghost" onClick={() => void onForget()}>
          Remove character
        </Button>
      </div>
      )}
    </>
  );
  if (embedded) return <div className="ft-charedit">{body}</div>;
  return <Card title="Your character">{body}</Card>;
}
