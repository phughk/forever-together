import { Button } from "../atoms/Button";
import { Notice } from "../atoms/Notice";
import { ConfirmAction } from "../molecules/ConfirmAction";
import { Locked } from "../molecules/Locked";
import { SettingRow } from "../molecules/SettingRow";

export interface HistorySettingsProps {
  onClear: () => void | Promise<void>;
  remote?: boolean;
  error?: string | null;
}

/** The "Search history" row with a Clear button. Sits first inside Advanced. */
export function HistorySettings({ onClear, remote, error }: HistorySettingsProps) {
  return (
    <>
      <SettingRow
        title="Search history"
        hint="A private record of your finished searches, kept only on this PC and used to give better advice. It is never shown or sent anywhere."
      >
        <Locked locked={remote}>
          {remote ? (
            <Button variant="danger" disabled>
              Clear
            </Button>
          ) : (
            <ConfirmAction
              label="Clear"
              confirmLabel="Yes, clear it"
              warning="Your search history is deleted for good. Advice may be less specific for a while."
              onConfirm={onClear}
            />
          )}
        </Locked>
      </SettingRow>
      {error && <Notice tone="bad">{error}</Notice>}
    </>
  );
}
