

import type { AddressBookListProps } from "../types";
import { sortAddressBookEntries } from "../lib";
import { AddressBookItem } from "./AddressBookItem";

export function AddressBookList({
  entries,
  loading = false,
  emptyMessage = "No address book entries yet.",
  onCopyAddress,
  onDelete,
}: AddressBookListProps) {
  const sortedEntries = sortAddressBookEntries(entries);

  if (loading && sortedEntries.length === 0) {
    return (
      <section className="address-book-list" aria-label="Address book entries">
        <div className="address-book-list__empty">Loading address book…</div>
      </section>
    );
  }

  if (sortedEntries.length === 0) {
    return (
      <section className="address-book-list" aria-label="Address book entries">
        <div className="address-book-list__empty">{emptyMessage}</div>
      </section>
    );
  }

  return (
    <section className="address-book-list" aria-label="Address book entries">
      {sortedEntries.map((entry) => (
        <AddressBookItem
          key={`${entry.wallet_name}:${entry.address}`}
          entry={entry}
          loading={loading}
          onCopyAddress={onCopyAddress}
          onDelete={onDelete}
        />
      ))}
    </section>
  );
}