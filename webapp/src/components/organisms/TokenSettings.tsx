import { Button } from "../atoms/Button";
import { Notice } from "../atoms/Notice";
import { ConfirmAction } from "../molecules/ConfirmAction";
import { Locked } from "../molecules/Locked";
import { SettingRow } from "../molecules/SettingRow";

export interface TokenSettingsProps {
  onReset: () => void | Promise<void>;
  remote?: boolean;
  error?: string | null;
}

/** The "Sign out all phones" row (the server call is still the token reset). Sits inside the Phone link card. */
export function TokenSettings({ onReset, remote, error }: TokenSettingsProps) {
  return (
    <>
      <SettingRow title="Sign out all phones">
        <Locked locked={remote}>
          {remote ? (
            <Button variant="danger" disabled>
              Sign out
            </Button>
          ) : (
            <ConfirmAction
              label="Sign out"
              confirmLabel="Yes, sign out"
              warning="Phones must scan the QR again."
              onConfirm={onReset}
            />
          )}
        </Locked>
      </SettingRow>
      {error && <Notice tone="bad">{error}</Notice>}
    </>
  );
}
