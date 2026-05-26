import { useMemo, useState } from "react";
import type { UtxosTableProps, UtxoSortKey, UtxoSortState } from "../types";
import {
  areAllVisibleUtxosSelected,
  areSomeVisibleUtxosSelected,
  getUtxoOutpoint,
  getUtxoValueSat,
  isUtxoLocked,
} from "../lib";
import {
  formatBtcFromSats,
  formatConfirmations,
  formatLockBadge,
  formatLockReason,
  formatOutpointFull,
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
  const [sort, setSort] = useState<UtxoSortState>({
    key: "value_sat",
    direction: "desc",
  });
  const selectionEnabled = Boolean(onToggleOutpoint);
  const allSelected = areAllVisibleUtxosSelected(utxos, selectedOutpoints);
  const someSelected = areSomeVisibleUtxosSelected(utxos, selectedOutpoints);
  const headerChecked = allSelected;

  const selectedSet = useMemo(() => new Set(selectedOutpoints), [selectedOutpoints]);

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
        return {
          key,
          direction: key === "value_sat" || key === "height" ? "desc" : "asc",
        };
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
        case "value_sat":
          return (getUtxoValueSat(a) - getUtxoValueSat(b)) * directionMultiplier;
        case "status":
          return (Number(a.confirmed) - Number(b.confirmed)) * directionMultiplier;
        case "lock_state":
          return (Number(isUtxoLocked(a)) - Number(isUtxoLocked(b))) * directionMultiplier;
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
              <button
                type="button"
                className="utxos-table__sort"
                onClick={() => handleSort("outpoint")}
              >
                Outpoint{sortLabel("outpoint")}
              </button>
            </th>
            <th>
              <button
                type="button"
                className="utxos-table__sort"
                onClick={() => handleSort("value_sat")}
              >
                Value{sortLabel("value_sat")}
              </button>
            </th>
            <th>
              <button
                type="button"
                className="utxos-table__sort"
                onClick={() => handleSort("status")}
              >
                Status{sortLabel("status")}
              </button>
            </th>
            <th>
              <button
                type="button"
                className="utxos-table__sort"
                onClick={() => handleSort("lock_state")}
              >
                Lock{sortLabel("lock_state")}
              </button>
            </th>
            <th>
              <button
                type="button"
                className="utxos-table__sort"
                onClick={() => handleSort("height")}
              >
                Height{sortLabel("height")}
              </button>
            </th>
            <th>
              <button
                type="button"
                className="utxos-table__sort"
                onClick={() => handleSort("keychain")}
              >
                Keychain{sortLabel("keychain")}
              </button>
            </th>
          </tr>
        </thead>
        <tbody>
          {sortedUtxos.map((utxo) => {
            const outpoint = getUtxoOutpoint(utxo);
            const vout = outpoint.split(":").at(-1);
            const isSelected = selectedSet.has(outpoint);
            const confirmations = utxo.confirmed ? 6 : 0;
            const confirmationLabel = utxo.confirmed ? "Confirmed" : "Pending";
            const valueSat = getUtxoValueSat(utxo);
            const isLocked = isUtxoLocked(utxo);
            const lockReason = formatLockReason(utxo.lock_reason);
            const lockTitle = isLocked
              ? `${lockReason} · locked at ${utxo.locked_at ?? "unknown"}`
              : "Available for spending";

            return (
              <tr
                key={outpoint}
                className={[isSelected ? "is-selected" : "", isLocked ? "is-locked" : ""]
                  .filter(Boolean)
                  .join(" ") || undefined}
                data-outpoint={outpoint}
                data-confirmed={Boolean(utxo.confirmed)}
                data-locked={isLocked}
              >
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
                  <div className="utxos-table__outpoint-cell">
                    <code title={formatOutpointFull(outpoint)}>
                      {formatOutpointShort(outpoint)}
                    </code>
                    <span>{vout ? `vout ${vout}` : "outpoint"}</span>
                  </div>
                </td>
                <td>
                  <div className="utxos-table__value-cell">
                    <strong className="utxos-table__value-primary">
                      {formatSats(valueSat)}
                    </strong>
                    <span className="utxos-table__value-secondary">
                      {formatBtcFromSats(valueSat)}
                    </span>
                  </div>
                </td>
                <td>
                  <div className="utxos-table__status-cell">
                    <span
                      className={`utxo-status ${utxo.confirmed ? "is-confirmed" : "is-pending"}`}
                      title={formatConfirmations(confirmations)}
                    >
                      {formatUtxoStatus(Boolean(utxo.confirmed))}
                    </span>
                    <span className="utxos-table__status-subvalue">
                      {confirmationLabel}
                    </span>
                  </div>
                </td>
                <td>
                  <div className="utxos-table__status-cell">
                    <span
                      className={`utxo-status ${isLocked ? "is-locked" : "is-spendable"}`}
                      title={lockTitle}
                    >
                      {formatLockBadge(isLocked)}
                    </span>
                    <span className="utxos-table__status-subvalue">
                      {isLocked ? lockReason : "Ready"}
                    </span>
                  </div>
                </td>
                <td title={formatConfirmations(confirmations)}>
                  <span className="utxos-table__height">
                    {utxo.confirmation_height ?? "—"}
                  </span>
                </td>
                <td>
                  <span className="utxos-table__keychain">
                    {utxo.keychain ?? "—"}
                  </span>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
