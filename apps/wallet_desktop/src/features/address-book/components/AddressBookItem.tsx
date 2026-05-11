import type { AddressBookItemProps } from "../types";
import {
  formatAddressBookAddress,
  formatAddressBookLabel,
  formatAddressBookNetwork,
  formatAddressBookNotes,
  formatAddressBookTimestamp,
} from "../format";

export function AddressBookItem({
  entry,
  loading = false,
  onCopyAddress,
  onDelete,
}: AddressBookItemProps) {
  async function handleDelete() {
    if (!onDelete || loading) {
      return;
    }

    await onDelete(entry);
  }

  function handleCopy() {
    void navigator.clipboard.writeText(entry.address);
    onCopyAddress?.(entry.address);
  }

  return (
    <article className="address-book-item">
      <div className="address-book-item__header">
        <div className="address-book-item__identity">
          <h3 className="address-book-item__label">
            {formatAddressBookLabel(entry.label)}
          </h3>

          <span className="address-book-item__network">
            {formatAddressBookNetwork(entry.network)}
          </span>
        </div>

        <div className="address-book-item__actions">
          <button
            className="address-book-item__button"
            type="button"
            disabled={loading}
            onClick={handleCopy}
          >
            Copy
          </button>

          <button
            className="address-book-item__button address-book-item__button--danger"
            type="button"
            disabled={loading}
            onClick={() => {
              void handleDelete();
            }}
          >
            Delete
          </button>
        </div>
      </div>

      <div className="address-book-item__content">
        <div className="address-book-item__row">
          <span className="address-book-item__meta-label">Address</span>

          <code className="address-book-item__address">
            {formatAddressBookAddress(entry.address)}
          </code>
        </div>

        <div className="address-book-item__row">
          <span className="address-book-item__meta-label">Notes</span>

          <span className="address-book-item__notes">
            {formatAddressBookNotes(entry.notes)}
          </span>
        </div>

        <div className="address-book-item__footer">
          <span>
            Created {formatAddressBookTimestamp(entry.created_at)}
          </span>

          {entry.updated_at ? (
            <span>
              Updated {formatAddressBookTimestamp(entry.updated_at)}
            </span>
          ) : null}
        </div>
      </div>
    </article>
  );
}
