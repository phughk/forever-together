import { useState } from "react";
import { Button } from "../atoms/Button";
import { Text } from "../atoms/Typography";

export interface ConfirmActionProps {
  label: string;
  /** The consequence in one sentence, shown before the second click. */
  warning: string;
  confirmLabel: string;
  onConfirm: () => void | Promise<void>;
  disabled?: boolean;
}

/** A destructive button that asks once more before it acts. */
export function ConfirmAction({ label, warning, confirmLabel, onConfirm, disabled }: ConfirmActionProps) {
  const [asking, setAsking] = useState(false);
  if (!asking) {
    return (
      <Button variant="danger" disabled={disabled} onClick={() => setAsking(true)}>
        {label}
      </Button>
    );
  }
  return (
    <div className="ft-confirm-action">
      <Text size="small">{warning}</Text>
      <div className="ft-confirm-action__row">
        <Button
          variant="danger"
          onClick={async () => {
            await onConfirm();
            setAsking(false);
          }}
        >
          {confirmLabel}
        </Button>
        <Button variant="ghost" onClick={() => setAsking(false)}>
          Cancel
        </Button>
      </div>
    </div>
  );
}
