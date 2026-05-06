import { useState } from "react";
import type { SweepFormProps, SweepFormState } from "../types";
import { sanitizeIntegerInput } from "../lib";

const initialForm: SweepFormState = {
  toAddress: "",
  feeRateSatPerVb: "",
  replaceable: true,
};

export function SweepForm({
  disabled = false,
  selectedUtxoCount = 0,
  onSubmit,
}: SweepFormProps) {
  const [form, setForm] = useState<SweepFormState>(initialForm);

  const feeRate = Number(form.feeRateSatPerVb);

  const hasUtxos = selectedUtxoCount > 0;

  const canSubmit =
    !disabled &&
    hasUtxos &&
    form.toAddress.trim().length > 0 &&
    Number.isFinite(feeRate) &&
    feeRate > 0;

  return (
    <section className="send-card">
      <div className="send-section-header">
        <div>
          <p className="send-eyebrow">Send</p>
          <h3>Sweep UTXOs</h3>
        </div>
      </div>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!canSubmit) return;

          onSubmit({
            ...form,
            toAddress: form.toAddress.trim(),
            feeRateSatPerVb: String(Number(form.feeRateSatPerVb)),
          });
        }}
        className="send-form-grid"
      >
        <div className="send-form-field">
          <label>Recipient address</label>
          <input
            type="text"
            value={form.toAddress}
            disabled={disabled}
            placeholder="bc1..."
            onChange={(e) =>
              setForm((prev) => ({ ...prev, toAddress: e.target.value }))
            }
          />
        </div>

        <div className="send-form-field">
          <label>Fee rate (sat/vB)</label>
          <input
            type="text"
            inputMode="numeric"
            value={form.feeRateSatPerVb}
            disabled={disabled}
            placeholder="e.g. 5"
            onChange={(e) =>
              setForm((prev) => ({
                ...prev,
                feeRateSatPerVb: sanitizeIntegerInput(e.target.value),
              }))
            }
          />
        </div>

        <div className="send-form-options">
          <label className="send-checkbox">
            <input
              type="checkbox"
              checked={form.replaceable}
              disabled={disabled}
              onChange={(e) =>
                setForm((prev) => ({ ...prev, replaceable: e.target.checked }))
              }
            />
            <span>Enable RBF (replace-by-fee)</span>
          </label>
        </div>

        <div className="send-helper-text">
          {!hasUtxos && (
            <span>Select at least one UTXO below to enable sweep.</span>
          )}
          {hasUtxos && (
            <span>{selectedUtxoCount} input(s) selected for sweep.</span>
          )}
        </div>

        <div className="send-form-actions">
          <button type="submit" disabled={!canSubmit}>
            Preview transaction
          </button>
        </div>
      </form>
    </section>
  );
}