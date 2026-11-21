import { useState } from "react";
import { STALE_CODE_AFTER_S } from "../../api/constants";
import type { CharacterView } from "../../api/types";
import { FACTION_LABEL, ROLE_LABEL, formatAge } from "../../lib/format";
import { Badge } from "../atoms/Badge";
import { Button } from "../atoms/Button";
import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { CodePasteForm } from "../molecules/CodePasteForm";
import { Field } from "../molecules/Field";

export interface CharacterPanelProps {
  character: CharacterView | null;
  /** Server-clock now in ms. */
  now: number;
  onSubmitCode: (code: string) => void | Promise<void>;
  onForget: () => void | Promise<void>;
  busy?: boolean;
  /** Plain-language reason the last pasted code was refused. */
  error?: string | null;
  /** The search is running, so the character cannot be replaced. */
  locked?: boolean;
}

export function CharacterPanel({ character, now, onSubmitCode, onForget, busy, error, locked }: CharacterPanelProps) {
  const [replacing, setReplacing] = useState(false);

  if (!character) {
    return (
      <Card title="Your character">
        <Text>
          Open the ForeverTogether addon in game with <b>/wowft</b>, copy the code with Ctrl+C and paste it here. On a phone, scan
          the addon's QR with the camera app and paste the text.
        </Text>
        <CodePasteForm onSubmit={onSubmitCode} busy={busy} error={error} />
      </Card>
    );
  }

  const ageS = (now - character.createdAt) / 1000;
  const stale = ageS > STALE_CODE_AFTER_S;
  const showForm = replacing || !!error;

  return (
    <Card title="Your character">
      <div>
        <Field label="Name">
          <b>
            {character.firstName} {character.surname}
          </b>
          <Badge tone={character.faction}>{FACTION_LABEL[character.faction]}</Badge>
        </Field>
        <Field label="Class and race">
          {[character.className, character.raceName].filter(Boolean).join(" ") || "Unknown"}
        </Field>
        <Field label="Level">{character.level}</Field>
        <Field label="Role">{ROLE_LABEL[character.role]}</Field>
        <Field label="Item level">{character.itemLevel > 0 ? character.itemLevel : "Unknown"}</Field>
        <Field label="Realm">{character.realmName}</Field>
        <Field label="Region">{character.regionName}</Field>
        <Field label="Zone">{character.zone || "Unknown"}</Field>
        <Field label="Code age">
          {formatAge(ageS)}
          {stale && <Badge tone="warn">Stale</Badge>}
        </Field>
      </div>
      {stale && (
        <Notice tone="warn">
          This code is {formatAge(ageS)} old. Level, gear or zone may have changed. Press Refresh in the addon window and paste
          the new code.
        </Notice>
      )}
      {!character.zoneKnown && (
        <Notice tone="info">
          The app does not know the zone{character.zone ? ` "${character.zone}"` : ""}, so no dungeon is listed first for being
          nearby. Every dungeon that fits your level is still offered.
        </Notice>
      )}
      <Text size="small" muted>
        Everything above is read from your code and is not verified.
      </Text>
      {showForm && <CodePasteForm onSubmit={async (c) => { await onSubmitCode(c); setReplacing(false); }} busy={busy} error={error} />}
      <div className="ft-actions">
        {!showForm && (
          <Button disabled={locked} onClick={() => setReplacing(true)}>
            Paste a new code
          </Button>
        )}
        {showForm && !error && (
          <Button variant="ghost" onClick={() => setReplacing(false)}>
            Cancel
          </Button>
        )}
        <Button variant="ghost" disabled={locked} onClick={() => void onForget()}>
          Forget this character
        </Button>
      </div>
      {locked && (
        <Text size="small" muted>
          Stop the search to change the character.
        </Text>
      )}
    </Card>
  );
}
