import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { useWallet } from "../app/providers/useWallet";
import { FixedSendForm } from "../features/send/components/FixedSendForm";
import { SendMaxForm } from "../features/send/components/SendMaxForm";
import { SweepForm } from "../features/send/components/SweepForm";
import { ConsolidationForm } from "../features/send/components/ConsolidationForm";
import { SendModeSelector } from "../features/send/components/SendModeSelector";
import { CoinControlSummary } from "../features/send/components/CoinControlSummary";
import { PsbtPreviewPanel } from "../features/send/components/PsbtPreviewPanel";
import {
  createConsolidationPsbt,
  createPsbt,
  createPsbtWithCoinControl,
  createSendMaxPsbt,
  createSendMaxPsbtWithCoinControl,
  createSweepPsbt,
  publishPsbt,
  signPsbt,
} from "../features/send/api";
import { listUtxos } from "../features/utxos/api";
import { syncWallet } from "../features/wallet/api";
import type {
  TxBroadcastResultDto,
  WalletPsbtDto,
  WalletSignedPsbtDto,
} from "../shared/types/dtos";
import type {
  CoinControlMode,
  CoinControlUtxoOption,
  FixedSendFormState,
  SendMaxFormState,
  SendMode,
  SendPageNavigationState,
  SweepFormState,
  ConsolidationFormState,
} from "../features/send/types";
import {
  buildSelectedCoinControl,
  buildSelectedConsolidation,
  findOutpointsForTxid,
  getValidSelectedOutpoints,
  mapUtxosForCoinControl,
  readReplaceableFromForm,
  sumSelectedInputValue,
} from "../features/send/lib";
import { formatNullableBoolean } from "../features/send/format";
import {
  toCreateConsolidationPsbtInput,
  toCreatePsbtInput,
  toCreateSendMaxPsbtInput,
  toCreateSweepPsbtInput,
} from "../features/send/types";


export function SendPage() {
  const { selectedWalletName } = useWallet();

  const location = useLocation();

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [psbt, setPsbt] = useState<WalletPsbtDto | null>(null);
  const [signedPsbt, setSignedPsbt] = useState<WalletSignedPsbtDto | null>(null);
  const [broadcastResult, setBroadcastResult] = useState<TxBroadcastResultDto | null>(null);
  const [coinSelectionMode, setCoinSelectionMode] = useState<CoinControlMode>("auto");
  const [availableUtxos, setAvailableUtxos] = useState<CoinControlUtxoOption[]>([]);
  const [selectedUtxos, setSelectedUtxos] = useState<string[]>([]);
  const [lastBroadcastCpfpOutpoints, setLastBroadcastCpfpOutpoints] = useState<string[]>([]);
  const [mode, setMode] = useState<SendMode>("fixed");

  const navigationState = location.state as SendPageNavigationState | null;

  useEffect(() => {
    if (!navigationState) return;

    if (
      navigationState.mode === "fixed" ||
      navigationState.mode === "send_max" ||
      navigationState.mode === "sweep" ||
      navigationState.mode === "consolidate"
    ) {
      setMode(navigationState.mode);

      if (navigationState.selectedOutpoints?.length) {
        setCoinSelectionMode("manual");
      }
    }

    if (navigationState.selectedOutpoints?.length) {
      setSelectedUtxos(navigationState.selectedOutpoints);
    }
  }, [location.key]);

  useEffect(() => {
    let cancelled = false;

    setError(null);
    setLastBroadcastCpfpOutpoints([]);

    if (!selectedWalletName) {
      setAvailableUtxos([]);
      setSelectedUtxos([]);
      return () => {
        cancelled = true;
      };
    }

    listUtxos(selectedWalletName)
      .then((utxos) => {
        if (cancelled) {
          return;
        }

        const mappedUtxos = mapUtxosForCoinControl(utxos);
        setAvailableUtxos(mappedUtxos);

        if (navigationState?.selectedOutpoints?.length) {
          setSelectedUtxos(getValidSelectedOutpoints(navigationState.selectedOutpoints, mappedUtxos));
          setCoinSelectionMode("manual");
        } else {
          setSelectedUtxos([]);
          setCoinSelectionMode("auto");
        }

        setLastBroadcastCpfpOutpoints([]);
      })
      .catch((e: unknown) => {
        if (cancelled) return;

        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        setAvailableUtxos([]);
        setSelectedUtxos([]);
        setCoinSelectionMode("auto");
      });

    return () => {
      cancelled = true;
    };
  }, [selectedWalletName, location.key]);

  useEffect(() => {
    const availableOutpoints = new Set(availableUtxos.map((utxo) => utxo.outpoint));

    setSelectedUtxos((current) =>
      current.filter((outpoint) => availableOutpoints.has(outpoint)),
    );
  }, [availableUtxos]);

  useEffect(() => {
    setPsbt(null);
    setSignedPsbt(null);
    setBroadcastResult(null);
    setLastBroadcastCpfpOutpoints([]);
  }, [coinSelectionMode, selectedUtxos]);

  useEffect(() => {
    if (coinSelectionMode === "auto") {
      setSelectedUtxos([]);
    }
  }, [coinSelectionMode]);

  useEffect(() => {
    setPsbt(null);
    setSignedPsbt(null);
    setBroadcastResult(null);
    setError(null);
    setLastBroadcastCpfpOutpoints([]);

    if (mode === "sweep" || mode === "consolidate") {
      setCoinSelectionMode("manual");
    }
  }, [mode]);

  useEffect(() => {
    setPsbt(null);
    setSignedPsbt(null);
    setBroadcastResult(null);
    setError(null);
    setLastBroadcastCpfpOutpoints([]);
  }, [selectedWalletName]);

  const selectedCoinControl = () => buildSelectedCoinControl(selectedUtxos, availableUtxos);
  const selectedConsolidation = () => buildSelectedConsolidation(selectedUtxos, availableUtxos);

  const preparePreview = () => {
    if (!selectedWalletName) return false;

    setLoading(true);
    setError(null);
    setPsbt(null);
    setSignedPsbt(null);
    setBroadcastResult(null);
    setLastBroadcastCpfpOutpoints([]);

    return true;
  };

  const handleFixedPreview = async (form: FixedSendFormState) => {
    if (!preparePreview() || !selectedWalletName) return;

    try {
      const input = {
        ...toCreatePsbtInput(form, selectedWalletName),
        replaceable: readReplaceableFromForm(form),
      };
      const selectedOutpoints = getValidSelectedOutpoints(selectedUtxos, availableUtxos);
      if (coinSelectionMode === "manual" && selectedOutpoints.length === 0) {
        throw new Error("Select at least one valid UTXO for manual coin control");
      }
      if (coinSelectionMode === "manual") {
        console.debug("[send/page] fixed manual coin-control preview", {
          walletName: input.walletName,
          toAddress: input.toAddress,
          amountSat: input.amountSat,
          feeRateSatPerVb: input.feeRateSatPerVb,
          replaceable: input.replaceable,
          confirmedOnly: Boolean(input.confirmedOnly),
          includeOutpoints: selectedOutpoints,
        });
      }
      const result =
        coinSelectionMode === "manual"
          ? await createPsbtWithCoinControl({
              walletName: input.walletName,
              toAddress: input.toAddress,
              amountSat: input.amountSat,
              feeRateSatPerVb: input.feeRateSatPerVb,
              replaceable: input.replaceable,
              confirmedOnly: Boolean(input.confirmedOnly),
              coinControl: {
                includeOutpoints: selectedOutpoints,
                excludeOutpoints: [],
                confirmedOnly: Boolean(input.confirmedOnly),
                selectionMode: "strict-manual",
              },
            })
          : await createPsbt(input);
      setPsbt(result);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const handleSendMaxPreview = async (form: SendMaxFormState) => {
    if (!preparePreview() || !selectedWalletName) return;

    try {
      const input = toCreateSendMaxPsbtInput(form, selectedWalletName);
      const selectedOutpoints = getValidSelectedOutpoints(selectedUtxos, availableUtxos);
      const result =
        coinSelectionMode === "manual" && selectedOutpoints.length > 0
          ? await createSendMaxPsbtWithCoinControl({
              ...input,
              coinControl: {
                includeOutpoints: selectedOutpoints,
                excludeOutpoints: [],
                confirmedOnly: true,
                selectionMode: "strict-manual",
              },
            })
          : await createSendMaxPsbt(input);

      setPsbt(result);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const handleSweepPreview = async (form: SweepFormState) => {
    if (!preparePreview() || !selectedWalletName) return;

    try {
      const coinControl = selectedCoinControl();
      const input = toCreateSweepPsbtInput(form, selectedWalletName, coinControl);
      const result = await createSweepPsbt(input);

      setPsbt(result);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const handleConsolidationPreview = async (form: ConsolidationFormState) => {
    if (!preparePreview() || !selectedWalletName) return;

    try {
      const consolidation = selectedConsolidation();
      const input = toCreateConsolidationPsbtInput(form, selectedWalletName, consolidation);
      const result = await createConsolidationPsbt(input);

      setPsbt(result);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const handleSign = async () => {
    if (!selectedWalletName || !psbt) return;

    setLoading(true);
    setError(null);
    setSignedPsbt(null);
    setBroadcastResult(null);
    setLastBroadcastCpfpOutpoints([]);

    try {
      const result = await signPsbt({
        walletName: selectedWalletName,
        psbtBase64: psbt.psbt_base64,
      });
      setSignedPsbt(result);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const handlePublish = async () => {
    if (!selectedWalletName || !signedPsbt) return;

    setLoading(true);
    setError(null);
    setBroadcastResult(null);

    try {
      const result = await publishPsbt({
        walletName: selectedWalletName,
        psbtBase64: signedPsbt.psbt_base64,
      });
      setBroadcastResult(result);

      try {
        await syncWallet(selectedWalletName);
      } catch (syncError: unknown) {
        console.warn("[send/page] sync after broadcast failed", syncError);
      }

      const refreshedUtxos = mapUtxosForCoinControl(await listUtxos(selectedWalletName));
      setAvailableUtxos(refreshedUtxos);
      setSelectedUtxos([]);
      const possibleCpfpOutpoints = findOutpointsForTxid(refreshedUtxos, result.txid);
      setLastBroadcastCpfpOutpoints(possibleCpfpOutpoints);
      console.debug("[send/page] possible CPFP outpoints after broadcast", {
        txid: result.txid,
        outpoints: possibleCpfpOutpoints,
      });
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const validSelectedUtxos = getValidSelectedOutpoints(selectedUtxos, availableUtxos);
  const isStrictManualMode = mode === "sweep" || mode === "consolidate";

  const manuallySelectedValueSat = sumSelectedInputValue(availableUtxos, validSelectedUtxos);

  const selectedInputCount =
    coinSelectionMode === "manual" ? validSelectedUtxos.length : psbt?.selected_utxo_count ?? 0;
  const selectedValueSat =
    coinSelectionMode === "manual"
      ? manuallySelectedValueSat
      : sumSelectedInputValue(availableUtxos, psbt?.selected_inputs ?? []);

  return (
    <section className="send-page">
      <header className="send-page__header send-page__header--compact">
        <div className="send-wallet-pill">
          Wallet: <span>{selectedWalletName || "—"}</span>
        </div>
      </header>

      {error && (
        <div className="send-page__error" role="alert">
          Error: {error}
        </div>
      )}

      <div className="send-page__stack">
        <SendModeSelector
          mode={mode}
          disabled={!selectedWalletName || loading}
          onModeChange={setMode}
        />

        {mode === "fixed" && (
          <div className="send-card">
            <FixedSendForm
              key="fixed"
              disabled={!selectedWalletName || loading}
              onSubmit={handleFixedPreview}
            />
          </div>
        )}

        {mode === "send_max" && (
          <SendMaxForm
            key="send_max"
            disabled={!selectedWalletName || loading}
            onSubmit={handleSendMaxPreview}
          />
        )}

        {mode === "sweep" && (
          <SweepForm
            key="sweep"
            disabled={!selectedWalletName || loading}
            selectedUtxoCount={validSelectedUtxos.length}
            onSubmit={handleSweepPreview}
          />
        )}

        {mode === "consolidate" && (
          <ConsolidationForm
            key="consolidate"
            disabled={!selectedWalletName || loading}
            selectedUtxoCount={validSelectedUtxos.length}
            onSubmit={handleConsolidationPreview}
          />
        )}

        <div className="send-card send-card--preview">
          <PsbtPreviewPanel psbt={psbt} />

          {psbt && (
            <div className="send-actions">
              <button
                className="primary-button"
                type="button"
                disabled={loading || !!signedPsbt}
                onClick={handleSign}
              >
                {signedPsbt ? "Signed" : "Sign PSBT"}
              </button>

              <button
                className="primary-button"
                type="button"
                disabled={loading || !signedPsbt || !signedPsbt.finalized || !!broadcastResult}
                onClick={handlePublish}
              >
                {broadcastResult ? "Broadcasted" : "Broadcast"}
              </button>

              {signedPsbt && !signedPsbt.finalized ? (
                <span className="send-actions__hint">
                  PSBT is signed but not finalized yet.
                </span>
              ) : null}
            </div>
          )}

          {signedPsbt && (
            <div className="send-result-card">
              <div className="send-result-card__label">Signed PSBT</div>
              <div className="send-result-card__value">
                finalized: {signedPsbt.finalized ? "yes" : "no"}
              </div>
              <div className="send-result-card__value">
                status: {signedPsbt.signing_status}
              </div>
            </div>
          )}

          {broadcastResult && (
            <div className="send-result-card send-result-card--success">
              <div className="send-result-card__label">Broadcast Result</div>
              <div className="send-result-card__value">txid: {broadcastResult.txid}</div>
              <div className="send-result-card__value">
                replaceable: {formatNullableBoolean(broadcastResult.replaceable)}
              </div>
              <div className="send-result-card__value">
                Possible CPFP outputs from refreshed UTXOs:{" "}
                {lastBroadcastCpfpOutpoints.length > 0 ? "available" : "not visible yet"}
              </div>
              {lastBroadcastCpfpOutpoints.length > 0 ? (
                <div className="send-result-card__value">
                  {lastBroadcastCpfpOutpoints.map((outpoint) => (
                    <div key={outpoint}>possible child input: {outpoint}</div>
                  ))}
                </div>
              ) : (
                <div className="send-result-card__value">
                  For CPFP, sync the wallet and open this transaction from Transactions. The details modal now shows inputs/outputs, and the CPFP panel reads wallet-owned outputs from the transaction row.
                </div>
              )}
            </div>
          )}
        </div>

        <div className="send-card">
          <CoinControlSummary
            selectedInputCount={selectedInputCount}
            selectedValueSat={selectedValueSat}
            utxos={availableUtxos}
            selectedUtxos={validSelectedUtxos}
            selectionMode={isStrictManualMode ? "manual" : coinSelectionMode}
            onSelectionModeChange={isStrictManualMode ? undefined : setCoinSelectionMode}
            onUtxoSelectionChange={setSelectedUtxos}
            onClearSelection={() => setSelectedUtxos([])}
          />
        </div>
      </div>
    </section>
  );
}
