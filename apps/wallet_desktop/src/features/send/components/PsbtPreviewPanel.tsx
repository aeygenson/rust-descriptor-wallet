import type { PsbtPreviewPanelProps } from "../types";
import {
  formatFeeRateSatPerVb,
  formatOptionalSats,
  formatOptionalValue,
  formatSelectedInput,
  formatVsize,
} from "../format";
import { pickDtoNumber, pickDtoString } from "../lib";

export function PsbtPreviewPanel({ psbt }: PsbtPreviewPanelProps) {
  if (!psbt) {
    return (
      <div className="send-preview-empty">
        <div className="send-preview-empty__title">PSBT Preview</div>
        <div className="send-preview-empty__text">
          Submit the form to generate a PSBT preview.
        </div>
      </div>
    );
  }

  const psbtRecord = psbt as unknown as Record<string, unknown>;
  const txid = pickDtoString(psbtRecord, "txid", "txid");
  const toAddress = pickDtoString(psbtRecord, "to_address", "toAddress");
  const amountSat = pickDtoNumber(psbtRecord, "amount_sat", "amountSat");
  const feeSat = pickDtoNumber(psbtRecord, "fee_sat", "feeSat");
  const feeRateSatPerVb = pickDtoNumber(
    psbtRecord,
    "fee_rate_sat_per_vb",
    "feeRateSatPerVb",
  );
  const estimatedVsize = pickDtoNumber(psbtRecord, "estimated_vsize", "estimatedVsize");
  const changeAmountSat = pickDtoNumber(
    psbtRecord,
    "change_amount_sat",
    "changeAmountSat",
  );
  const inputCount = pickDtoNumber(psbtRecord, "input_count", "inputCount");
  const outputCount = pickDtoNumber(psbtRecord, "output_count", "outputCount");
  const selectedInputs = Array.isArray(psbtRecord.selected_inputs)
    ? psbtRecord.selected_inputs
    : Array.isArray(psbtRecord.selectedInputs)
      ? psbtRecord.selectedInputs
      : [];
  const psbtBase64 = pickDtoString(psbtRecord, "psbt_base64", "psbtBase64") ?? "";

  return (
    <div className="send-preview">
      <div className="send-preview__header">
        <h3 className="send-preview__title">PSBT Preview</h3>
        <div className="send-preview__badge">
          {psbt.replaceable ? "RBF enabled" : "Final tx"}
        </div>
      </div>

      <div className="send-preview-grid">
        <PreviewItem label="Txid" mono value={formatOptionalValue(txid)} />
        <PreviewItem label="To" mono value={formatOptionalValue(toAddress)} />

        <PreviewItem label="Amount" value={formatOptionalSats(amountSat)} />
        <PreviewItem label="Fee" value={formatOptionalSats(feeSat)} />
        <PreviewItem
          label="Fee rate"
          value={formatFeeRateSatPerVb(feeRateSatPerVb)}
        />
        <PreviewItem
          label="vsize"
          value={formatVsize(estimatedVsize)}
        />
        <PreviewItem label="Change" value={formatOptionalSats(changeAmountSat)} />
        <PreviewItem
          label="Inputs / Outputs"
          value={`${formatOptionalValue(inputCount)} / ${formatOptionalValue(outputCount)}`}
        />
      </div>

      <div className="send-preview-section">
        <div className="send-preview-section__title">Selected Inputs</div>

        {selectedInputs?.length > 0 ? (
          <div className="send-preview-inputs">
            {selectedInputs.map((input, index) => {
              const text = formatSelectedInput(input);

              return (
                <div
                  key={`${text}-${index}`}
                  className="send-preview-input"
                  title={text}
                >
                  {text}
                </div>
              );
            })}
          </div>
        ) : (
          <div className="send-preview-empty-inline">
            No selected inputs reported
          </div>
        )}
      </div>

      <details className="send-preview-raw">
        <summary>Raw PSBT</summary>
        <pre>{psbtBase64}</pre>
      </details>
    </div>
  );
}

function PreviewItem({
  label,
  value,
  mono = false,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="send-preview-item">
      <div className="send-preview-item__label">{label}</div>
      <div
        className={
          mono
            ? "send-preview-item__value send-preview-item__value--mono"
            : "send-preview-item__value"
        }
      >
        {value}
      </div>
    </div>
  );
}