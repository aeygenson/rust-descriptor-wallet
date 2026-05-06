import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useWallet } from "../app/providers/useWallet";
import { listUtxos } from "../features/utxos/api";
import { UtxoActionsBar } from "../features/utxos/components/UtxoActionsBar";
import { UtxoSelectionSummary } from "../features/utxos/components/UtxoSelectionSummary";
import { UtxosHeader } from "../features/utxos/components/UtxosHeader";
import { UtxosStateView } from "../features/utxos/components/UtxosStateView";
import { UtxosTable } from "../features/utxos/components/UtxosTable";
import {
  buildUtxoSelectionSummary,
  getUtxoValueSat,
  getValidSelectedOutpoints,
  selectAllVisibleOutpoints,
  toggleSelectedOutpoint,
} from "../features/utxos/lib";
import type {
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
    }

    return {
      totalCount: utxos.length,
      totalValue,
      averageValue: utxos.length > 0 ? Math.round(totalValue / utxos.length) : 0,
      confirmedCount,
      confirmedValue,
      pendingCount,
      pendingValue,
      keychains: Array.from(keychainSet).join(", ") || "—",
    };
  }, [utxos]);

  const validSelectedOutpoints = useMemo(
    () => getValidSelectedOutpoints(utxos, selectedOutpoints),
    [utxos, selectedOutpoints]
  );

  const selectionSummary = useMemo(
    () => buildUtxoSelectionSummary(utxos, validSelectedOutpoints),
    [utxos, validSelectedOutpoints]
  );

  const handleToggleOutpoint = (outpoint: UtxoOutpoint) => {
    setSelectedOutpoints((current) => toggleSelectedOutpoint(current, outpoint));
  };

  const handleSelectAllVisible = () => {
    setSelectedOutpoints(selectAllVisibleOutpoints(utxos));
  };

  const handleClearSelection = () => {
    setSelectedOutpoints([]);
  };

  const navigateToSendWithMode = (mode: "fixed" | "send_max" | "sweep" | "consolidate") => {
    if (validSelectedOutpoints.length === 0) return;

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

      <UtxosStateView loading={loading} error={error} hasData={utxos.length > 0} />

      {utxos.length > 0 && (
        <>
          <UtxoSelectionSummary
            selectedCount={selectionSummary.selectedCount}
            selectedValueSat={selectionSummary.selectedValueSat}
            confirmedCount={selectionSummary.confirmedCount}
            unconfirmedCount={selectionSummary.unconfirmedCount}
            onClearSelection={handleClearSelection}
          />

          <UtxoActionsBar
            selectedCount={selectionSummary.selectedCount}
            selectedValueSat={selectionSummary.selectedValueSat}
            disabled={loading || selectionSummary.selectedCount === 0}
            onSendFixedSelected={() => navigateToSendWithMode("fixed")}
            onSendMaxSelected={() => navigateToSendWithMode("send_max")}
            onSweepSelected={() => navigateToSendWithMode("sweep")}
            onConsolidateSelected={handleConsolidateSelected}
            onClearSelection={handleClearSelection}
          />

          <UtxosTable
            utxos={utxos}
            selectedOutpoints={validSelectedOutpoints}
            onToggleOutpoint={handleToggleOutpoint}
            onSelectAllVisible={handleSelectAllVisible}
            onClearSelection={handleClearSelection}
          />
        </>
      )}
    </section>
  );
}