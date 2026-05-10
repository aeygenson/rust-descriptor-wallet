
import type { ReceiveAddressActionsProps } from "../types";


export function ReceiveAddressActions({
  address,
  bitcoinUri,
  loading = false,
  onRefresh,
  onCopyAddress,
  onCopyBitcoinUri,
}: ReceiveAddressActionsProps) {
  const hasAddress = address.trim().length > 0;
  const hasBitcoinUri = bitcoinUri.trim().length > 0;

  return (
    <div className="receive-address-actions">
      <button
        className="receive-address-actions__button"
        type="button"
        disabled={!hasAddress || loading}
        onClick={() => onCopyAddress?.(address)}
      >
        Copy address
      </button>

      <button
        className="receive-address-actions__button"
        type="button"
        disabled={!hasBitcoinUri || loading}
        onClick={() => onCopyBitcoinUri?.(bitcoinUri)}
      >
        Copy URI
      </button>

      {onRefresh ? (
        <button
          className="receive-address-actions__button receive-address-actions__button--primary"
          type="button"
          disabled={loading}
          onClick={onRefresh}
        >
          {loading ? "Generating…" : "Generate new"}
        </button>
      ) : null}
    </div>
  );
}