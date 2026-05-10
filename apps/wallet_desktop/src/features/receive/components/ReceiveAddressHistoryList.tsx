

import type { ReceiveAddressHistoryListProps } from "../types";

import { ReceiveAddressHistoryItem } from "./ReceiveAddressHistoryItem";

export function ReceiveAddressHistoryList({
  walletName,
  addresses,
  loading = false,
  onSelect,
}: ReceiveAddressHistoryListProps) {
  if (loading && addresses.length === 0) {
    return (
      <section className="receive-history-list" aria-label="Receive address history">
        <div className="receive-history-item">
          <div className="receive-history-item__address">Loading receive history…</div>
        </div>
      </section>
    );
  }

  if (addresses.length === 0) {
    return (
      <section className="receive-history-list" aria-label="Receive address history">
        <div className="receive-history-item">
          <div className="receive-history-item__address">
            No receive addresses generated yet.
          </div>
        </div>
      </section>
    );
  }

  return (
    <section className="receive-history-list" aria-label="Receive address history">
      {addresses.map((address) => (
        <ReceiveAddressHistoryItem
          key={address.address}
          walletName={walletName}
          address={address}
          onSelect={onSelect}
        />
      ))}
    </section>
  );
}