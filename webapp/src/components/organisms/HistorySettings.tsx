import { Card } from "../atoms/Card";
import { Notice } from "../atoms/Notice";
import { Text } from "../atoms/Typography";
import { ConfirmAction } from "../molecules/ConfirmAction";

export interface HistorySettingsProps {
  onClear: () => void | Promise<void>;
  remote?: boolean;
  error?: string | null;
}

export function HistorySettings({ onClear, remote, error }: HistorySettingsProps) {
  return (
    <Card title="Search history">
      <Text size="small" muted>
        This app keeps a private record of your finished searches on this computer. It never shows it and never sends it
        anywhere; it only uses it to give you better advice.
      </Text>
      {remote ? (
        <Notice tone="info">Change this on the desktop app.</Notice>
      ) : (
        <ConfirmAction
          label="Clear search history"
          confirmLabel="Yes, clear it"
          warning="The record of your finished searches will be deleted for good. Advice may be less specific until new searches are recorded."
          onConfirm={onClear}
        />
      )}
      {error && <Notice tone="bad">{error}</Notice>}
    </Card>
  );
}
