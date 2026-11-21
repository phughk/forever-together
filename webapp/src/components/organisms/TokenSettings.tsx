import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { ConfirmAction } from "../molecules/ConfirmAction";

export interface TokenSettingsProps {
  onReset: () => void | Promise<void>;
  remote?: boolean;
  error?: string | null;
}

export function TokenSettings({ onReset, remote, error }: TokenSettingsProps) {
  return (
    <Card title="Paired phones">
      <Text size="small" muted>
        Each phone holds a copy of this install's token. Resetting it signs out every phone at once. Lost a phone? Reset, then
        pair the others again.
      </Text>
      {remote ? (
        <Notice tone="info">The token can be reset only in the desktop app.</Notice>
      ) : (
        <ConfirmAction
          label="Reset token"
          confirmLabel="Yes, sign out all phones"
          warning="Every paired phone will stop working until it scans the pairing QR again. This window gets the new token automatically."
          onConfirm={onReset}
        />
      )}
      {error && <Notice tone="bad">{error}</Notice>}
    </Card>
  );
}
