

import type { ReceiveQrCodeProps } from "../types";

export function ReceiveQrCode({
  value,
  size = 220,
}: ReceiveQrCodeProps) {
  const normalizedValue = value.trim();
  const hasValue = normalizedValue.length > 0;

  return (
    <div
      className="receive-qr"
      style={{
        width: `${size}px`,
        height: `${size}px`,
      }}
      aria-label="Receive QR code"
    >
      {hasValue ? (
        <div className="receive-qr__placeholder">
          <div className="receive-qr__grid" />

          <div className="receive-qr__label">
            QR preview
          </div>
        </div>
      ) : (
        <div className="receive-qr__empty">
          No QR data
        </div>
      )}
    </div>
  );
}