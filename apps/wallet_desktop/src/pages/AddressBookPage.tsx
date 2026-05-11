

import { useCallback, useEffect, useState } from "react";

import type { AddressBookEntryDto } from "../shared/types/dtos";
import type { AddressBookFormValues } from "../features/address-book/types";
import {
  createAddressBookEntry,
  deleteAddressBookEntry,
  listAddressBookEntries,
} from "../features/address-book/api";
import {
  normalizeAddressBookAddress,
  normalizeAddressBookLabel,
  normalizeAddressBookNotes,
} from "../features/address-book/lib";
import { AddressBookForm } from "../features/address-book/components/AddressBookForm";
import { AddressBookList } from "../features/address-book/components/AddressBookList";
import { useWallet } from "../app/providers/useWallet";

export function AddressBookPage() {
  const { selectedWalletName } = useWallet();
  const [entries, setEntries] = useState<AddressBookEntryDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const walletName = selectedWalletName ?? "";

  const loadEntries = useCallback(async () => {
    if (!walletName) {
      setEntries([]);
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const result = await listAddressBookEntries({ name: walletName });
      setEntries(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [walletName]);

  useEffect(() => {
    void loadEntries();
  }, [loadEntries]);

  async function handleCreate(values: AddressBookFormValues) {
    if (!walletName) {
      setError("No wallet selected");
      return;
    }

    try {
      setLoading(true);
      setError(null);

      await createAddressBookEntry({
        name: walletName,
        label: normalizeAddressBookLabel(values.label),
        address: normalizeAddressBookAddress(values.address),
        notes: normalizeAddressBookNotes(values.notes),
      });

      await loadEntries();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete(entry: AddressBookEntryDto) {
    if (!walletName) {
      setError("No wallet selected");
      return;
    }

    try {
      setLoading(true);
      setError(null);

      await deleteAddressBookEntry({
        name: walletName,
        address: entry.address,
      });

      await loadEntries();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  function handleCopyAddress(address: string) {
    void navigator.clipboard.writeText(address);
  }

  return (
    <section className="address-book-page">
      <header className="address-book-page__header">
        <div>
          <h1 className="address-book-page__title">Address Book</h1>

          <p className="address-book-page__subtitle">
            Store wallet-scoped destination addresses so you do not accidentally
            send to the wrong network.
          </p>
        </div>

        <div className="address-book-page__wallet-pill">
          {walletName || "No wallet selected"}
        </div>
      </header>

      {error ? (
        <div className="address-book-page__error" role="alert">
          {error}
        </div>
      ) : null}

      <section className="address-book-page__panel">
        <div className="address-book-page__section-header">
          <h2 className="address-book-page__section-title">Add destination</h2>
          <p className="address-book-page__section-subtitle">
            Entries are persisted per wallet and tagged with that wallet network.
          </p>
        </div>

        <AddressBookForm
          walletName={walletName}
          loading={loading}
          onSubmit={handleCreate}
        />
      </section>

      <section className="address-book-page__panel">
        <div className="address-book-page__section-header">
          <h2 className="address-book-page__section-title">Saved addresses</h2>
          <p className="address-book-page__section-subtitle">
            {entries.length} saved {entries.length === 1 ? "entry" : "entries"}.
          </p>
        </div>

        <AddressBookList
          entries={entries}
          loading={loading}
          onCopyAddress={handleCopyAddress}
          onDelete={handleDelete}
        />
      </section>
    </section>
  );
}