

import type { ReceiveAddressMetadataProps } from "../types";

import { formatAddressIndex } from "../format";

export function ReceiveAddressMetadata({
  items,
}: ReceiveAddressMetadataProps) {
  if (items.length === 0) {
    return null;
  }

  return (
    <dl className="receive-address-metadata">
      {items.map((item) => {
        const value =
          item.label === "Index"
            ? formatAddressIndex(
                typeof item.value === "number" ? item.value : null,
              )
            : item.value;

        return (
          <div
            key={item.label}
            className="receive-address-metadata__item"
          >
            <dt>{item.label}</dt>
            <dd>{value}</dd>
          </div>
        );
      })}
    </dl>
  );
}