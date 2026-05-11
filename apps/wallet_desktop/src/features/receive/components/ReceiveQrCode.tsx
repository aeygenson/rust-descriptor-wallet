import type { ReceiveQrCodeProps } from "../types";

export function ReceiveQrCode({
  value,
  svg,
  size = 220,
}: ReceiveQrCodeProps) {
  const normalizedValue = value.trim();
  const normalizedSvg = svg?.trim() ?? "";
  const hasSvg = normalizedSvg.length > 0;

  return (
    <div
      className="receive-qr"
      style={{
        width: `${size}px`,
        height: `${size}px`,
      }}
      aria-label="Receive QR code"
      title={normalizedValue || "Receive QR code"}
    >
      {hasSvg ? (
        <img
          className="receive-qr__image"
          src={`data:image/svg+xml;utf8,${encodeURIComponent(normalizedSvg)}`}
          alt="Bitcoin receive QR code"
        />
      ) : (
        <div className="receive-qr__empty">
          No QR data
        </div>
      )}
    </div>
  );
}