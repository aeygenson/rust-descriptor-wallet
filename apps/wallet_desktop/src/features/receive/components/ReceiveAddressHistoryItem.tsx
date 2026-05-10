

import type { ReceiveAddressHistoryItemProps } from "../types";

import {
  formatAddressIndex,
  formatReceiveAddress,
  formatReceiveLabel,
  formatReceiveTimestamp,
} from "../format";

export function ReceiveAddressHistoryItem({
  walletName,
  address,
  selected = false,
  onSelect,
}: ReceiveAddressHistoryItemProps) {
  const addressLabel = formatReceiveLabel(address.label);
  const addressIndex = formatAddressIndex(address.index);
  const createdAt = formatReceiveTimestamp(address.created_at);

  return (
    <button
      className={[
        "receive-history-item",
        selected ? "receive-history-item--selected" : "",
      ]
        .filter(Boolean)
        .join(" ")}
      type="button"
      onClick={() => onSelect?.(address)}
      aria-label={`Select receive address ${address.address} for wallet ${walletName}`}
    >
      <div className="receive-history-item__address">
        {formatReceiveAddress(address.address)}
      </div>

      <div className="receive-history-item__meta">
        <span className="receive-history-item__badge">
          {address.keychain}
        </span>

        <span className="receive-history-item__badge">
          index {addressIndex}
        </span>

        {addressLabel !== "—" ? (
          <span className="receive-history-item__badge">
            {addressLabel}
          </span>
        ) : null}

        <span className="receive-history-item__badge">
          {createdAt}
        </span>
      </div>
    </button>
  );
}