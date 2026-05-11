

import { useState } from "react";

import type { AddressBookFormProps, AddressBookFormValues } from "../types";
import { isValidAddressBookForm } from "../lib";

const EMPTY_VALUES: AddressBookFormValues = {
  label: "",
  address: "",
  notes: "",
};

export function AddressBookForm({
  walletName,
  loading = false,
  onSubmit,
}: AddressBookFormProps) {
  const [values, setValues] = useState<AddressBookFormValues>(EMPTY_VALUES);
  const canSubmit = walletName.trim().length > 0 && isValidAddressBookForm(values) && !loading;

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!canSubmit) {
      return;
    }

    await onSubmit(values);
    setValues(EMPTY_VALUES);
  }

  return (
    <form className="address-book-form" onSubmit={handleSubmit}>
      <div className="address-book-form__grid">
        <label className="address-book-form__field">
          <span className="address-book-form__label">Label</span>
          <input
            className="address-book-form__input"
            type="text"
            value={values.label}
            placeholder="Exchange, cold storage, friend…"
            disabled={loading}
            onChange={(event) =>
              setValues((current) => ({
                ...current,
                label: event.target.value,
              }))
            }
          />
        </label>

        <label className="address-book-form__field">
          <span className="address-book-form__label">Address</span>
          <input
            className="address-book-form__input"
            type="text"
            value={values.address}
            placeholder="Bitcoin destination address"
            disabled={loading}
            onChange={(event) =>
              setValues((current) => ({
                ...current,
                address: event.target.value,
              }))
            }
          />
        </label>

        <label className="address-book-form__field address-book-form__field--full">
          <span className="address-book-form__label">Notes</span>
          <textarea
            className="address-book-form__textarea"
            value={values.notes}
            placeholder="Optional notes"
            disabled={loading}
            rows={3}
            onChange={(event) =>
              setValues((current) => ({
                ...current,
                notes: event.target.value,
              }))
            }
          />
        </label>
      </div>

      <div className="address-book-form__actions">
        <button
          className="address-book-form__button address-book-form__button--primary"
          type="submit"
          disabled={!canSubmit}
        >
          {loading ? "Saving…" : "Save address"}
        </button>
      </div>
    </form>
  );
}