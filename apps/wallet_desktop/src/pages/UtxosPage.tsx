import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useWallet } from "../app/providers/useWallet";
import { listUtxos, lockUtxo, unlockUtxo } from "../features/utxos/api";
import { UtxoActionsBar } from "../features/utxos/components/UtxoActionsBar";
import { UtxoSelectionSummary } from "../features/utxos/components/UtxoSelectionSummary";
import { UtxosHeader } from "../features/utxos/components/UtxosHeader";
import { UtxosStateView } from "../features/utxos/components/UtxosStateView";
import { UtxosTable } from "../features/utxos/components/UtxosTable";
import {
  buildUtxoSelectionPreview,
  buildUtxoSelectionSummary,
  filterUtxos,
  getLockedSelectedOutpoints,
  getSpendableSelectedOutpoints,
  getUtxoValueSat,
  getValidSelectedOutpoints,
  isUtxoLocked,
  selectAllVisibleOutpoints,
  toggleSelectedOutpoint,
} from "../features/utxos/lib";
import type {
  UtxoFilterStatus,
  UtxoOutpoint,
  UtxosPageNavigationActionState,
  UtxosSummary,
} from "../features/utxos/types";
import type { WalletUtxoDto } from "../shared/types/dtos";

export function UtxosPage() {
  const navigate = useNavigate();
  const { selectedWalletName } = useWallet();
  const [utxos, setUtxos] = useState<WalletUtxoDto[]>([]);
  const [selectedOutpoints, setSelectedOutpoints] = useState<UtxoOutpoint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterStatus, setFilterStatus] = useState<UtxoFilterStatus>("all");

  useEffect(() => {
    let cancelled = false;

    if (!selectedWalletName) {
      setUtxos([]);
      setSelectedOutpoints([]);
      return () => {
        cancelled = true;
      };
    }

    setLoading(true);
    setError(null);

    listUtxos(selectedWalletName)
      .then((data) => {
        if (!cancelled) {
          setUtxos(data);
          setSelectedOutpoints((current) => getValidSelectedOutpoints(data, current));
        }
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          const msg = e instanceof Error ? e.message : String(e);
          setError(msg);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedWalletName]);

  useEffect(() => {
    setSelectedOutpoints([]);
  }, [selectedWalletName]);

  const summary = useMemo<UtxosSummary>(() => {
    let totalValue = 0;
    let confirmedValue = 0;
    let pendingValue = 0;
    let confirmedCount = 0;
    let pendingCount = 0;
    let lockedValue = 0;
    let spendableValue = 0;
    let lockedCount = 0;
    let spendableCount = 0;
    const keychainSet = new Set<string>();

    for (const utxo of utxos) {
      const value = getUtxoValueSat(utxo);
      totalValue += value;

      if (utxo.keychain) {
        keychainSet.add(utxo.keychain);
      }

      if (utxo.confirmed) {
        confirmedCount += 1;
        confirmedValue += value;
      } else {
        pendingCount += 1;
        pendingValue += value;
      }

      if (isUtxoLocked(utxo)) {
        lockedCount += 1;
        lockedValue += value;
      } else {
        spendableCount += 1;
        spendableValue += value;
      }
    }

    return {
      totalCount: utxos.length,
      totalValue,
      averageValue: utxos.length > 0 ? Math.round(totalValue / utxos.length) : 0,
      confirmedCount,
      confirmedValue,
      pendingCount,
      pendingValue,
      lockedCount,
      lockedValue,
      spendableCount,
      spendableValue,
      keychains: Array.from(keychainSet).join(", ") || "—",
    };
  }, [utxos]);

  const visibleUtxos = useMemo(
    () =>
      filterUtxos(utxos, {
        status: filterStatus,
      }),
    [utxos, filterStatus]
  );

  const hasWalletUtxos = utxos.length > 0;
  const hasVisibleUtxos = visibleUtxos.length > 0;
  const activeFilterLabel = filterStatus === "all" ? "All" : filterStatus;

  const filterCounts = useMemo<Record<UtxoFilterStatus, number>>(
    () => ({
      all: utxos.length,
      confirmed: utxos.filter((utxo) => utxo.confirmed).length,
      pending: utxos.filter((utxo) => !utxo.confirmed).length,
      locked: utxos.filter(isUtxoLocked).length,
      spendable: utxos.filter((utxo) => !isUtxoLocked(utxo)).length,
    }),
    [utxos]
  );

  const validSelectedOutpoints = useMemo(
    () => getValidSelectedOutpoints(visibleUtxos, selectedOutpoints),
    [visibleUtxos, selectedOutpoints]
  );

  const selectionSummary = useMemo(
    () => buildUtxoSelectionSummary(utxos, validSelectedOutpoints),
    [utxos, validSelectedOutpoints]
  );

  const selectionPreview = useMemo(
    () => buildUtxoSelectionPreview(visibleUtxos, validSelectedOutpoints),
    [visibleUtxos, validSelectedOutpoints]
  );

  const lockedSelectedOutpoints = useMemo(
    () => getLockedSelectedOutpoints(utxos, validSelectedOutpoints),
    [utxos, validSelectedOutpoints]
  );

  const spendableSelectedOutpoints = useMemo(
    () => getSpendableSelectedOutpoints(utxos, validSelectedOutpoints),
    [utxos, validSelectedOutpoints]
  );

  const hasLockedSelection = lockedSelectedOutpoints.length > 0;
  const hasSpendableSelection = spendableSelectedOutpoints.length > 0;

  const handleToggleOutpoint = (outpoint: UtxoOutpoint) => {
    setSelectedOutpoints((current) => toggleSelectedOutpoint(current, outpoint));
  };

  const handleSelectAllVisible = () => {
    setSelectedOutpoints(selectAllVisibleOutpoints(visibleUtxos));
  };

  const handleClearSelection = () => {
    setSelectedOutpoints([]);
  };

  const reloadUtxos = async (walletName: string) => {
    const data = await listUtxos(walletName);
    setUtxos(data);
    setSelectedOutpoints((current) => getValidSelectedOutpoints(data, current));
  };

  const handleLockSelected = async () => {
    if (!selectedWalletName || spendableSelectedOutpoints.length === 0) return;

    setLoading(true);
    setError(null);

    try {
      await Promise.all(
        spendableSelectedOutpoints.map((outpoint) =>
          lockUtxo(selectedWalletName, outpoint, "Locked from desktop UTXO page")
        )
      );
      await reloadUtxos(selectedWalletName);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const handleUnlockSelected = async () => {
    if (!selectedWalletName || lockedSelectedOutpoints.length === 0) return;

    setLoading(true);
    setError(null);

    try {
      await Promise.all(
        lockedSelectedOutpoints.map((outpoint) => unlockUtxo(selectedWalletName, outpoint))
      );
      await reloadUtxos(selectedWalletName);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const navigateToSendWithMode = (mode: "fixed" | "send_max" | "sweep" | "consolidate") => {
    if (validSelectedOutpoints.length === 0) return;
    if (hasLockedSelection) {
      setError("Selected UTXOs include locked coins. Unlock them before spending.");
      return;
    }

    const state: UtxosPageNavigationActionState = {
      mode,
      selectedOutpoints: validSelectedOutpoints,
    };

    navigate("/send", { state });
  };

  const handleConsolidateSelected = () => {
    navigateToSendWithMode("consolidate");
  };

  return (
    <section className="utxos-page">
      <UtxosHeader walletName={selectedWalletName} summary={summary} />

      <UtxosStateView loading={loading} error={error} hasData={hasWalletUtxos} />

      {hasWalletUtxos && (
        <div
          className="utxos-filter-bar"
          data-active-filter={filterStatus}
          data-visible-count={visibleUtxos.length}
        >
          <div className="utxos-filter-bar__left">
            <span className="utxos-filter-bar__label">
              Filter
              <strong>{activeFilterLabel}</strong>
            </span>
            <div
              className="utxos-filter-bar__buttons"
              role="group"
              aria-label="UTXO status filters"
            >
              {(["all", "confirmed", "pending", "locked", "spendable"] as UtxoFilterStatus[]).map((status) => (
                <button
                  key={status}
                  type="button"
                  aria-pressed={filterStatus === status}
                  title={`Show ${status} UTXOs`}
                  data-filter={status}
                  className={
                    filterStatus === status
                      ? "utxos-filter-bar__button is-active"
                      : "utxos-filter-bar__button"
                  }
                  onClick={() => setFilterStatus(status)}
                >
                  <span>{status}</span>
                  <strong>{filterCounts[status].toLocaleString()}</strong>
                </button>
              ))}
            </div>
          </div>
          <span className="utxos-filter-bar__meta">
            Showing <strong>{visibleUtxos.length.toLocaleString()}</strong> /{" "}
            {utxos.length.toLocaleString()}
          </span>
        </div>
      )}

      {hasVisibleUtxos ? (
        <>
          <UtxoSelectionSummary
            selectedCount={selectionPreview.selectedCount}
            selectedValueSat={selectionPreview.selectedValueSat}
            confirmedCount={selectionSummary.confirmedCount}
            unconfirmedCount={selectionSummary.unconfirmedCount}
            lockedCount={selectionSummary.lockedCount}
            spendableCount={selectionSummary.spendableCount}
            onClearSelection={handleClearSelection}
          />

          <UtxoActionsBar
            selectedCount={selectionPreview.selectedCount}
            selectedValueSat={selectionPreview.selectedValueSat}
            hasLockedSelection={hasLockedSelection}
            hasSpendableSelection={hasSpendableSelection}
            disabled={loading || selectionPreview.selectedCount === 0}
            onSendFixedSelected={() => navigateToSendWithMode("fixed")}
            onSendMaxSelected={() => navigateToSendWithMode("send_max")}
            onSweepSelected={() => navigateToSendWithMode("sweep")}
            onConsolidateSelected={handleConsolidateSelected}
            onLockSelected={handleLockSelected}
            onUnlockSelected={handleUnlockSelected}
            onClearSelection={handleClearSelection}
          />

          <UtxosTable
            utxos={visibleUtxos}
            selectedOutpoints={validSelectedOutpoints}
            onToggleOutpoint={handleToggleOutpoint}
            onSelectAllVisible={handleSelectAllVisible}
            onClearSelection={handleClearSelection}
          />
        </>
      ) : hasWalletUtxos ? (
        <div className="utxos-filter-empty">
          <strong>No UTXOs match the current filter</strong>
          <span>
            Switch back to All or choose another status filter to view wallet
            outputs.
          </span>
          <button type="button" onClick={() => setFilterStatus("all")}>
            Show all UTXOs
          </button>
        </div>
      ) : null}
    </section>
  );
}