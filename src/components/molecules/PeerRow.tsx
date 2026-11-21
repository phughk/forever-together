import { Button } from "../atoms/Button";

export interface PeerRowProps {
  peerKey: string;
  onRemove?: (key: string) => void;
}

export function PeerRow({ peerKey, onRemove }: PeerRowProps) {
  return (
    <div className="ft-peer">
      <span className="ft-peer__key">{peerKey}</span>
      {onRemove && (
        <Button size="small" variant="ghost" aria-label={`Remove ${peerKey}`} onClick={() => onRemove(peerKey)}>
          Remove
        </Button>
      )}
    </div>
  );
}
