import type { ReceiveAddressCardProps } from "../types";

import {
  formatAddressIndex,
  formatReceiveAddress,
} from "../format";
import {
  buildBitcoinUri,
  getReceiveAddressMetadata,
  getReceiveAddressString,
} from "../lib";

export function ReceiveAddressCard({
  walletName,
  address,
  loading = false,
  onRefresh,
  onCopy,
}: ReceiveAddressCardProps) {
  const addressString = getReceiveAddressString(address);
  const formattedAddress = formatReceiveAddress(addressString);
  const metadata = getReceiveAddressMetadata(address);
  const bitcoinUri = buildBitcoinUri(address);

  async function handleCopy(): Promise<void> {
    if (!addressString) {
      return;
    }

    await navigator.clipboard.writeText(addressString);
    onCopy?.(addressString);
  }

  async function handleCopyUri(): Promise<void> {
    if (!bitcoinUri) {
      return;
    }

    await navigator.clipboard.writeText(bitcoinUri);
  }

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
        <dl className="receive-card__metadata">
          {metadata.map((item) => (
            <div
              key={item.label}
              className="receive-card__metadata-item"
            >
              <dt>{item.label}</dt>
              <dd>
                {item.label === "Index"
                  ? formatAddressIndex(
                      typeof item.value === "number"
                        ? item.value
                        : null,
                    )
                  : item.value}
              </dd>
            </div>
          ))}
        </dl>
      ) : null}

      <div className="receive-card__actions">
        <button
          className="receive-card__button"
          type="button"
          disabled={!addressString}
          onClick={() => {
            void handleCopy();
          }}
        >
          Copy address
        </button>

        <button
          className="receive-card__button"
          type="button"
          disabled={!bitcoinUri}
          onClick={() => {
            void handleCopyUri();
          }}
        >
          Copy bitcoin URI
        </button>
      </div>
    </section>
  );
}
