

import { useEffect, useState } from "react";

import type { ReceiveAddressLabelEditorProps } from "../types";

export function ReceiveAddressLabelEditor({
  address,
  loading = false,
  onSave,
  onClear,
}: ReceiveAddressLabelEditorProps) {
  const [label, setLabel] = useState(address.label ?? "");

  useEffect(() => {
    setLabel(address.label ?? "");
  }, [address.address, address.label]);

  const normalizedLabel = label.trim();
  const currentLabel = address.label?.trim() ?? "";
  const canSave = normalizedLabel.length > 0 && normalizedLabel !== currentLabel;
  const canClear = currentLabel.length > 0;

  return (
    <div className="receive-label-editor">
      <label className="receive-label-editor__label" htmlFor="receive-address-label">
        Label
      </label>

      <div className="receive-label-editor__row">
        <input
          id="receive-address-label"
          className="receive-label-editor__input"
          type="text"
          value={label}
          placeholder="Optional receive address label"
          disabled={loading}
          onChange={(event) => setLabel(event.target.value)}
        />

        <button
          className="receive-label-editor__button receive-label-editor__button--primary"
          type="button"
          disabled={loading || !canSave}
          onClick={() => onSave?.(normalizedLabel)}
        >
          Save
        </button>

        <button
          className="receive-label-editor__button"
          type="button"
          disabled={loading || !canClear}
          onClick={onClear}
        >
          Clear
        </button>
      </div>
    </div>
  );
}