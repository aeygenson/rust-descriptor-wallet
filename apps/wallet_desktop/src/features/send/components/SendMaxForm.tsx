import { useState } from "react";
import type { SendMaxFormProps, SendMaxFormState } from "../types";
import { sanitizeIntegerInput } from "../lib";

const initialForm: SendMaxFormState = {
  toAddress: "",
  feeRateSatPerVb: "",
  replaceable: true,
};

export function SendMaxForm({ disabled = false, onSubmit }: SendMaxFormProps) {
  const [form, setForm] = useState<SendMaxFormState>(initialForm);

  const feeRate = Number(form.feeRateSatPerVb);

  const canSubmit =
    !disabled &&
    form.toAddress.trim().length > 0 &&
    Number.isFinite(feeRate) &&
    feeRate > 0;

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

        <div className="send-form-actions">
          <button type="submit" disabled={!canSubmit}>
            Preview transaction
          </button>
        </div>
      </form>
    </section>
  );
}