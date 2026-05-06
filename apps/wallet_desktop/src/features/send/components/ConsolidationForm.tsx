import { useState } from "react";
import type { ConsolidationFormProps, ConsolidationFormState } from "../types";
import { sanitizeIntegerInput } from "../lib";

const initialForm: ConsolidationFormState = {
  feeRateSatPerVb: "",
  replaceable: true,
};

export function ConsolidationForm({
  disabled = false,
  selectedUtxoCount = 0,
  onSubmit,
}: ConsolidationFormProps) {
  const [form, setForm] = useState<ConsolidationFormState>(initialForm);

  const feeRate = Number(form.feeRateSatPerVb);
  const hasEnoughUtxos = selectedUtxoCount >= 2;

  const canSubmit =
    !disabled &&
    hasEnoughUtxos &&
    Number.isFinite(feeRate) &&
    feeRate > 0;

  return (
    <section className="send-card">
      <div className="send-section-header">
        <div>
          <p className="send-eyebrow">UTXO management</p>
          <h3>Consolidate UTXOs</h3>
        </div>
      </div>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!canSubmit) return;

          onSubmit({
            ...form,
            feeRateSatPerVb: String(Number(form.feeRateSatPerVb)),
          });
        }}
        className="send-form-grid"
      >
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
          {!hasEnoughUtxos && (
            <span>Select at least two UTXOs to enable consolidation.</span>
          )}
          {hasEnoughUtxos && (
            <span>
              {selectedUtxoCount} input(s) selected. Consolidation creates a new wallet-controlled output.
            </span>
          )}
        </div>

        <div className="send-form-actions">
          <button type="submit" disabled={!canSubmit}>
            Preview consolidation
          </button>
        </div>
      </form>
    </section>
  );
}