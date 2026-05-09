import { useState } from "react";
import type { SendMaxFormProps, SendMaxFormState } from "../types";
import { sanitizeIntegerInput } from "../lib";

const initialForm: SendMaxFormState = {
  toAddress: "",
  feeRateSatPerVb: "",
  replaceable: true,
};

export function SendMaxForm({
  disabled = false,
  onSubmit,
}: SendMaxFormProps) {
  const [form, setForm] = useState<SendMaxFormState>(initialForm);

  const feeRate = Number(form.feeRateSatPerVb);
  const normalizedFeeRateSatPerVb = String(feeRate);
  const normalizedAddress = form.toAddress.trim();

  const hasAddress = normalizedAddress.length > 0;
  const hasValidFeeRate = Number.isFinite(feeRate) && feeRate > 0;

  const estimatedSpendAll = hasAddress && hasValidFeeRate;

  const canSubmit = !disabled && hasAddress && hasValidFeeRate;

  const submitTitle = canSubmit
    ? "Preview an unsigned Send Max PSBT"
    : "Enter recipient address and positive fee rate";

  return (
    <section className="send-card">
      <div className="send-section-header">
        <div>
          <p className="send-eyebrow">Send</p>
          <h3>Send max</h3>
        </div>
      </div>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!canSubmit) {
            return;
          }

          onSubmit({
            ...form,
            toAddress: normalizedAddress,
            feeRateSatPerVb: normalizedFeeRateSatPerVb,
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
            aria-invalid={form.toAddress.length > 0 && !hasAddress}
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
            aria-invalid={form.feeRateSatPerVb.length > 0 && !hasValidFeeRate}
            onChange={(e) =>
              setForm((prev) => ({
                ...prev,
                feeRateSatPerVb: sanitizeIntegerInput(e.target.value),
              }))
            }
          />
          {form.feeRateSatPerVb.length > 0 && !hasValidFeeRate ? (
            <span className="field__error">
              Enter a positive fee rate in sat/vB.
            </span>
          ) : null}
        </div>

        <div className="send-form-options">
          <label className="send-checkbox">
            <input
              type="checkbox"
              checked={form.replaceable}
              disabled={disabled}
              onChange={(e) =>
                setForm((prev) => ({
                  ...prev,
                  replaceable: e.target.checked,
                }))
              }
            />
            <span>Enable RBF (replace-by-fee)</span>
          </label>
        </div>

        <div className="send-helper-text">
          <span>
            Send Max spends the available wallet funds minus network fees.
            Preview the PSBT first to verify the final output amount, selected
            inputs, change behavior, and RBF status before signing.
          </span>
        </div>

        {estimatedSpendAll ? (
          <div className="send-helper-text">
            <span>
              The backend will automatically calculate the maximum spendable
              amount after subtracting estimated mining fees.
            </span>
          </div>
        ) : null}

        <div className="send-form-actions">
          <button type="submit" disabled={!canSubmit} title={submitTitle}>
            Preview transaction
          </button>
        </div>
      </form>
    </section>
  );
}