import type { ReceiveAddressCardProps } from "../types";

import { formatReceiveAddress } from "../format";
import {
  buildBitcoinUri,
  getReceiveAddressMetadata,
  getReceiveAddressString,
  getReceiveQrSvg,
} from "../lib";

import { ReceiveAddressActions } from "./ReceiveAddressActions";
import { ReceiveAddressLabelEditor } from "./ReceiveAddressLabelEditor";
import { ReceiveAddressMetadata } from "./ReceiveAddressMetadata";
import { ReceiveQrCode } from "./ReceiveQrCode";

export function ReceiveAddressCard({
  walletName,
  address,
  loading = false,
  onRefresh,
  onCopy,
  onSaveLabel,
  onClearLabel,
}: ReceiveAddressCardProps) {
  const addressString = getReceiveAddressString(address);
  const formattedAddress = formatReceiveAddress(addressString);
  const metadata = getReceiveAddressMetadata(address);
  const bitcoinUri = buildBitcoinUri(address);
  const qrSvg = getReceiveQrSvg(address);


  return (
    <section className="receive-card">
      <header className="receive-card__header">
        <div>
          <p className="receive-card__eyebrow">Receive Bitcoin</p>
          <h2 className="receive-card__title">{walletName}</h2>
        </div>

        <button
          className="receive-card__refresh"
          type="button"
          disabled={loading}
          onClick={onRefresh}
        >
          {loading ? "Refreshing…" : "Generate new"}
        </button>
      </header>

      <div className="receive-card__address-block">
        <span className="receive-card__address-label">Address</span>

        <code
          className="receive-card__address"
          title={addressString}
        >
          {formattedAddress}
        </code>
      </div>

      {metadata.length > 0 ? (
        <ReceiveAddressMetadata items={metadata} />
      ) : null}

      <ReceiveQrCode
        value={bitcoinUri}
        svg={qrSvg}
      />

      <ReceiveAddressLabelEditor
        address={address}
        loading={loading}
        onSave={onSaveLabel}
        onClear={onClearLabel}
      />

      <ReceiveAddressActions
        address={addressString}
        bitcoinUri={bitcoinUri}
        loading={loading}
        onRefresh={onRefresh}
        onCopyAddress={(value) => {
          void navigator.clipboard.writeText(value);
          onCopy?.(value);
        }}
        onCopyBitcoinUri={(value) => {
          void navigator.clipboard.writeText(value);
        }}
      />
    </section>
  );
}
