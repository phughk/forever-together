import qrcode from "qrcode-generator";
import { useMemo } from "react";

export interface QrCodeProps {
  value: string;
  /** Accessible description of what the code opens. */
  label: string;
}

const QUIET_ZONE = 4;

/** Draws a QR as one SVG path, error correction level M, with a four-module quiet zone. */
export function QrCode({ value, label }: QrCodeProps) {
  const { size, path } = useMemo(() => {
    const qr = qrcode(0, "M");
    qr.addData(value);
    qr.make();
    const n = qr.getModuleCount();
    let d = "";
    for (let r = 0; r < n; r++) {
      for (let c = 0; c < n; c++) {
        if (qr.isDark(r, c)) d += `M${c + QUIET_ZONE} ${r + QUIET_ZONE}h1v1h-1z`;
      }
    }
    return { size: n + QUIET_ZONE * 2, path: d };
  }, [value]);

  return (
    <svg className="ft-qr" viewBox={`0 0 ${size} ${size}`} role="img" aria-label={label} shapeRendering="crispEdges">
      <rect width={size} height={size} fill="#fff" />
      <path d={path} fill="#000" />
    </svg>
  );
}
