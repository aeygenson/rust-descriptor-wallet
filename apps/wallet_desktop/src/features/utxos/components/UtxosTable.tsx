import { useMemo, useState } from "react";
import type { UtxosTableProps, UtxoSortKey, UtxoSortState } from "../types";
import {
  areAllVisibleUtxosSelected,
  areSomeVisibleUtxosSelected,
  getUtxoOutpoint,
  getUtxoValueSat,
} from "../lib";
import {
  formatOutpointShort,
  formatSats,
  formatUtxoStatus,
} from "../format";


export function UtxosTable({
  utxos,
  selectedOutpoints = [],
  onToggleOutpoint,
  onSelectAllVisible,
  onClearSelection,
}: UtxosTableProps) {
  const [sort, setSort] = useState<UtxoSortState>({ key: "value", direction: "desc" });
  const selectionEnabled = Boolean(onToggleOutpoint);
  const allSelected = areAllVisibleUtxosSelected(utxos, selectedOutpoints);
  const someSelected = areSomeVisibleUtxosSelected(utxos, selectedOutpoints);
  const headerChecked = allSelected;

  const handleHeaderToggle = () => {
    if (!selectionEnabled) return;

    if (allSelected || someSelected) {
      onClearSelection?.();
    } else {
      onSelectAllVisible?.();
    }
  };

  const handleSort = (key: UtxoSortKey) => {
    setSort((current) => {
      if (current.key !== key) {
        return { key, direction: key === "value" || key === "height" ? "desc" : "asc" };
      }

      return {
        key,
        direction: current.direction === "asc" ? "desc" : "asc",
      };
    });
  };

  const sortLabel = (key: UtxoSortKey) => {
    if (sort.key !== key) return "";
    return sort.direction === "asc" ? " ↑" : " ↓";
  };

  const sortedUtxos = useMemo(() => {
    const directionMultiplier = sort.direction === "asc" ? 1 : -1;

    return [...utxos].sort((a, b) => {
      switch (sort.key) {
        case "outpoint":
          return getUtxoOutpoint(a).localeCompare(getUtxoOutpoint(b)) * directionMultiplier;
        case "value":
          return (getUtxoValueSat(a) - getUtxoValueSat(b)) * directionMultiplier;
        case "status":
          return (Number(a.confirmed) - Number(b.confirmed)) * directionMultiplier;
        case "height": {
          const aHeight = a.confirmation_height ?? -1;
          const bHeight = b.confirmation_height ?? -1;
          return (aHeight - bHeight) * directionMultiplier;
        }
        case "keychain":
          return String(a.keychain ?? "").localeCompare(String(b.keychain ?? "")) * directionMultiplier;
        default:
          return 0;
      }
    });
  }, [sort, utxos]);

  return (
    <div className="utxos-table-wrap">
      <table className="utxos-table">
        <thead>
          <tr>
            {selectionEnabled && (
              <th className="utxos-table__checkbox-cell">
                <input
                  type="checkbox"
                  aria-label="Select all visible UTXOs"
                  checked={headerChecked}
                  ref={(input) => {
                    if (input) input.indeterminate = someSelected && !allSelected;
                  }}
                  onChange={handleHeaderToggle}
                />
              </th>
            )}
            <th>
              <button type="button" className="utxos-table__sort" onClick={() => handleSort("outpoint")}>
                Outpoint{sortLabel("outpoint")}
              </button>
            </th>
            <th>
              <button type="button" className="utxos-table__sort" onClick={() => handleSort("value")}>
                Value{sortLabel("value")}
              </button>
            </th>
            <th>
              <button type="button" className="utxos-table__sort" onClick={() => handleSort("status")}>
                Status{sortLabel("status")}
              </button>
            </th>
            <th>
              <button type="button" className="utxos-table__sort" onClick={() => handleSort("height")}>
                Height{sortLabel("height")}
              </button>
            </th>
            <th>
              <button type="button" className="utxos-table__sort" onClick={() => handleSort("keychain")}>
                Keychain{sortLabel("keychain")}
              </button>
            </th>
          </tr>
        </thead>
        <tbody>
          {sortedUtxos.map((utxo) => {
            const outpoint = getUtxoOutpoint(utxo);
            const isSelected = selectedOutpoints.includes(outpoint);
            const valueSat = getUtxoValueSat(utxo);

            return (
              <tr key={outpoint} className={isSelected ? "is-selected" : undefined}>
                {selectionEnabled && (
                  <td className="utxos-table__checkbox-cell">
                    <input
                      type="checkbox"
                      aria-label={`Select UTXO ${outpoint}`}
                      checked={isSelected}
                      onChange={() => onToggleOutpoint?.(outpoint)}
                    />
                  </td>
                )}
                <td>
                  <code title={outpoint}>{formatOutpointShort(outpoint)}</code>
                </td>
                <td>{formatSats(valueSat)}</td>
                <td>
                  <span className={`utxo-status ${utxo.confirmed ? "is-confirmed" : "is-pending"}`}>
                    {formatUtxoStatus(Boolean(utxo.confirmed))}
                  </span>
                </td>
                <td>{utxo.confirmation_height ?? "—"}</td>
                <td>{utxo.keychain ?? "—"}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
