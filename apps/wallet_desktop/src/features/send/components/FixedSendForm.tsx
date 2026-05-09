import { useState } from "react";
import type { FixedSendFormProps, FixedSendFormState } from "../types";
import { sanitizeDecimalInput, sanitizeIntegerInput } from "../lib";

const initialForm: FixedSendFormState = {
  toAddress: "",
  amountSat: "",
  feeRateSatPerVb: "1",
  replaceable: false,
  confirmedOnly: true,
};
export function FixedSendForm({
  disabled = false,
  onSubmit,
}: FixedSendFormProps) {
  const [form, setForm] = useState<FixedSendFormState>(initialForm);

  const updateField = <K extends keyof FixedSendFormState>(
    key: K,
    value: FixedSendFormState[K],
  ) => {
    setForm((current) => ({ ...current, [key]: value }));
  };

  const amountSat = Number(form.amountSat);
  const feeRateSatPerVb = Number(form.feeRateSatPerVb);
  const normalizedAddress = form.toAddress.trim();

  const hasAddress = normalizedAddress.length > 0;
  const hasValidAmount = Number.isFinite(amountSat) && amountSat > 0;
  const hasValidFeeRate =
    Number.isFinite(feeRateSatPerVb) && feeRateSatPerVb > 0;

  const canSubmit =
    !disabled &&
    hasAddress &&
    hasValidAmount &&
    hasValidFeeRate;

  const submitTitle = canSubmit
    ? "Preview an unsigned fixed-amount PSBT"
    : "Enter destination address, amount, and positive fee rate";

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (!canSubmit) {
      return;
    }

    onSubmit({
      ...form,
      toAddress: normalizedAddress,
      amountSat: String(amountSat),
      feeRateSatPerVb: String(feeRateSatPerVb),
    });
  };

  return (
    <form className="send-form" onSubmit={handleSubmit}>
      <div className="send-form__header">
        <h3 className="send-form__title">Fixed Amount Send</h3>
      </div>

      <div className="send-form__grid">
        <label className="field field--wide">
          <span className="field__label">Destination address</span>
          <input
            className="field__input"
            value={form.toAddress}
            onChange={(event) => updateField("toAddress", event.target.value)}
            placeholder="bc1... / tb1... / bcrt1..."
            maxLength={120}
            disabled={disabled}
            aria-invalid={form.toAddress.length > 0 && !hasAddress}
          />
        </label>

        <label className="field">
          <span className="field__label">Amount (sats)</span>
          <input
            className="field__input"
            value={form.amountSat}
            onChange={(event) =>
              updateField("amountSat", sanitizeIntegerInput(event.target.value))
            }
            placeholder="10000"
            inputMode="numeric"
            type="text"
            pattern="[0-9]*"
            autoComplete="off"
            aria-describedby="amount-sats-help"
            disabled={disabled}
            aria-invalid={form.amountSat.length > 0 && !hasValidAmount}
          />
          <span id="amount-sats-help" className="field__hint">
            Whole satoshis only.
          </span>
          {form.amountSat.length > 0 && !hasValidAmount ? (
            <span className="field__error">Enter a positive satoshi amount.</span>
          ) : null}
        </label>

        <label className="field">
          <span className="field__label">Fee rate (sat/vB)</span>
          <input
            className="field__input"
            value={form.feeRateSatPerVb}
            onChange={(event) =>
              updateField(
                "feeRateSatPerVb",
                sanitizeDecimalInput(event.target.value),
              )
            }
            placeholder="1.5"
            inputMode="decimal"
            type="text"
            pattern="[0-9]*[.]?[0-9]*"
            autoComplete="off"
            aria-describedby="fee-rate-help"
            disabled={disabled}
            aria-invalid={form.feeRateSatPerVb.length > 0 && !hasValidFeeRate}
          />
          <span id="fee-rate-help" className="field__hint">
            Decimal values are allowed, for example 1.5.
          </span>
          {form.feeRateSatPerVb.length > 0 && !hasValidFeeRate ? (
            <span className="field__error">
              Enter a positive fee rate in sat/vB.
            </span>
          ) : null}
        </label>
      </div>

      <div className="send-form__options">
        <label className="checkbox">
          <input
            type="checkbox"
            checked={form.replaceable}
            onChange={(event) =>
              updateField("replaceable", event.target.checked)
            }
            disabled={disabled}
          />
          <span>RBF enabled</span>
        </label>
        <label className="checkbox">
          <input
            type="checkbox"
            checked={form.confirmedOnly}
            onChange={(event) =>
              updateField("confirmedOnly", event.target.checked)
            }
            disabled={disabled}
          />
          <span>Use only confirmed UTXOs</span>
        </label>
      </div>
      <div className="send-helper-text">
        <span>
          Preview builds an unsigned PSBT first. Signing and publishing stay
          separate so you can inspect fees, inputs, change, and RBF status
          before broadcasting.
        </span>
      </div>
      <div className="send-form__actions">
        <button
          className="primary-button"
          type="submit"
          disabled={!canSubmit}
          title={submitTitle}
        >
          Preview PSBT
        </button>
      </div>
    </form>
  );
}