import type { PsbtPreviewPanelProps } from "../types";
import {
  formatBtc,
  formatFeeRateSatPerVb,
  formatOptionalSats,
  formatOptionalValue,
  formatOutpoint,
  formatSelectedInput,
  formatSelectedInputWithBtc,
  formatTxid,
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
  const estimatedVsize = pickDtoNumber(
    psbtRecord,
    "estimated_vsize",
    "estimatedVsize",
  );
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
  const psbtBase64 =
    pickDtoString(psbtRecord, "psbt_base64", "psbtBase64") ?? "";

  const amountBtc = formatBtc(amountSat);
  const feeBtc = formatBtc(feeSat);
  const changeBtc = formatBtc(changeAmountSat);

  const replacementRecord =
    typeof psbtRecord.replacement === "object" &&
    psbtRecord.replacement !== null
      ? (psbtRecord.replacement as Record<string, unknown>)
      : null;
  const replacedTxid = replacementRecord
    ? pickDtoString(replacementRecord, "replaced_txid", "replacedTxid")
    : null;
  const replacementTxid = replacementRecord
    ? pickDtoString(replacementRecord, "replacement_txid", "replacementTxid")
    : null;
  const replacementDepth = replacementRecord
    ? pickDtoNumber(replacementRecord, "replacement_depth", "replacementDepth")
    : null;
  const replacementChain = replacementRecord
    ? Array.isArray(replacementRecord.replacement_chain)
      ? replacementRecord.replacement_chain
      : Array.isArray(replacementRecord.replacementChain)
        ? replacementRecord.replacementChain
        : []
    : [];

  return (
    <div className="send-preview">
      <div className="send-preview__header">
        <h3 className="send-preview__title">PSBT Preview</h3>
        <div className="send-preview__badge">
          {psbt.replaceable ? "RBF enabled" : "Final tx"}
        </div>
      </div>

      <div className="send-preview-grid">
        <PreviewItem
          label="Txid"
          mono
          title={txid ?? undefined}
          value={txid ? formatTxid(txid) : formatOptionalValue(txid)}
        />
        <PreviewItem label="To" mono value={formatOptionalValue(toAddress)} />
        <PreviewItem
          label="Amount"
          value={formatOptionalSats(amountSat)}
          secondaryValue={amountBtc}
        />
        <PreviewItem
          label="Fee"
          value={formatOptionalSats(feeSat)}
          secondaryValue={feeBtc}
        />
        <PreviewItem
          label="Fee rate"
          value={formatFeeRateSatPerVb(feeRateSatPerVb)}
        />
        <PreviewItem label="vsize" value={formatVsize(estimatedVsize)} />
        <PreviewItem
          label="Change"
          value={formatOptionalSats(changeAmountSat)}
          secondaryValue={changeBtc}
        />
        <PreviewItem
          label="Inputs / Outputs"
          value={[
            formatOptionalValue(inputCount),
            formatOptionalValue(outputCount),
          ].join(" / ")}
        />
      </div>

      <div className="send-preview-section">
        <div className="send-preview-section__title">Selected Inputs</div>

        {selectedInputs?.length > 0 ? (
          <div className="send-preview-inputs">
            {selectedInputs.map((input, index) => {
              const text = formatSelectedInput(input);
              const displayText =
                typeof input === "string"
                  ? formatOutpoint(input)
                  : formatSelectedInputWithBtc(
                      typeof input === "number" ? input : null,
                    );

              return (
                <div
                  key={`${text}-${index}`}
                  className="send-preview-input"
                  title={text}
                >
                  {displayText}
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

      {replacementRecord ? (
        <div className="send-preview-section">
          <div className="send-preview-section__title">Replacement</div>
          <div className="send-preview-grid">
            <PreviewItem
              label="Replaces"
              mono
              title={replacedTxid ?? undefined}
              value={replacedTxid ? formatTxid(replacedTxid) : "—"}
            />
            <PreviewItem
              label="Replacement txid"
              mono
              title={replacementTxid ?? undefined}
              value={replacementTxid ? formatTxid(replacementTxid) : "—"}
            />
            <PreviewItem
              label="Depth"
              value={formatOptionalValue(replacementDepth)}
            />
            <PreviewItem
              label="Chain length"
              value={formatOptionalValue(replacementChain.length)}
            />
          </div>
        </div>
      ) : null}

      <details className="send-preview-raw">
        <summary>Raw PSBT</summary>
        <pre>{psbtBase64}</pre>
      </details>
    </div>
  );
}

type PreviewItemProps = {
  label: string;
  value: string;
  secondaryValue?: string;
  mono?: boolean;
  title?: string;
};

function PreviewItem({
  label,
  value,
  secondaryValue,
  mono = false,
  title,
}: PreviewItemProps) {
  return (
    <div className="send-preview-item">
      <div className="send-preview-item__label">{label}</div>
      <div
        title={title}
        className={
          mono
            ? "send-preview-item__value send-preview-item__value--mono"
            : "send-preview-item__value"
        }
      >
        {value}
        {secondaryValue ? (
          <span className="send-preview-item__secondary-value">
            {secondaryValue}
          </span>
        ) : null}
      </div>
    </div>
  );
}