import { useEffect, useState } from "react";
import { getAppInfo, getBackendHealth, getWalletStatus } from "../features/wallet/api";
import type { WalletBackendHealthDto, WalletStatusDto } from "../shared/types/dtos";
import { useWallet } from "../app/providers/useWallet";
import { formatOptionalSats } from "../features/send/format";

export function OverviewPage() {
  const [backendInfo, setBackendInfo] = useState("Connecting to backend...");
  const [status, setStatus] = useState<WalletStatusDto | null>(null);
  const [statusLoading, setStatusLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [backendHealth, setBackendHealth] = useState<WalletBackendHealthDto | null>(null);
  const [backendHealthLoading, setBackendHealthLoading] = useState(false);
  const [backendHealthError, setBackendHealthError] = useState<string | null>(null);

  const { selectedWalletName } = useWallet();

  const backendHealthOk = backendHealth
    ? backendHealth.sync_backend_reachable &&
      backendHealth.bitcoin_tip_reachable &&
      backendHealth.broadcast_backend_reachable
    : false;

  useEffect(() => {
    let isMounted = true;

    const loadAppInfo = async () => {
      try {
        const info = await getAppInfo();

        if (isMounted) {
          setBackendInfo(info);
        }
      } catch (e: unknown) {
        if (isMounted) {
          const msg = e instanceof Error ? e.message : String(e);
          setBackendInfo("Failed to connect to backend");
          setError(msg);
        }
      }
    };

    loadAppInfo();

    return () => {
      isMounted = false;
    };
  }, []);

  useEffect(() => {
    let isMounted = true;

    const loadStatus = async () => {
      if (!selectedWalletName) {
        if (isMounted) {
          setStatus(null);
          setStatusLoading(false);
        }
        return;
      }

      try {
        if (isMounted) {
          setStatusLoading(true);
          setError(null);
        }

        const walletStatus = await getWalletStatus(selectedWalletName);

        if (isMounted) {
          setStatus(walletStatus);
        }
      } catch (e: unknown) {
        if (isMounted) {
          const msg = e instanceof Error ? e.message : String(e);
          setStatus(null);
          setError(msg);
        }
      } finally {
        if (isMounted) {
          setStatusLoading(false);
        }
      }
    };

    loadStatus();

    return () => {
      isMounted = false;
    };
  }, [selectedWalletName]);

  useEffect(() => {
    let isMounted = true;

    const loadBackendHealth = async () => {
      if (!selectedWalletName) {
        if (isMounted) {
          setBackendHealth(null);
          setBackendHealthLoading(false);
          setBackendHealthError(null);
        }
        return;
      }

      try {
        if (isMounted) {
          setBackendHealthLoading(true);
          setBackendHealthError(null);
        }

        const health = await getBackendHealth(selectedWalletName);

        if (isMounted) {
          setBackendHealth(health);
        }
      } catch (e: unknown) {
        if (isMounted) {
          const msg = e instanceof Error ? e.message : String(e);
          setBackendHealth(null);
          setBackendHealthError(msg);
        }
      } finally {
        if (isMounted) {
          setBackendHealthLoading(false);
        }
      }
    };

    loadBackendHealth();

    return () => {
      isMounted = false;
    };
  }, [selectedWalletName]);

  return (
    <section className="overview-page">
      <div className="overview-grid">
        <section className="overview-card overview-card--hero">
          <div className="overview-card__icon" aria-hidden="true">
            ▣
          </div>
          <div>
            <h3 className="overview-card__title">Backend</h3>
            <p className="overview-card__subtitle">
              <span className="status-dot" aria-hidden="true" />
              {backendInfo}
            </p>
            <p className="overview-card__subtitle">
              Wallet: {selectedWalletName ?? "none selected"}
            </p>
          </div>
        </section>

        <section className="overview-card">
          <div className="overview-card__header">
            <div className="overview-card__icon" aria-hidden="true">
              ◎
            </div>
            <h3 className="overview-card__title">Bitcoin Backend Health</h3>
          </div>

          <div className="overview-card__body">
            {backendHealthLoading ? (
              <span className="overview-empty">Checking backend health...</span>
            ) : backendHealth ? (
              <div className="overview-health">
                <div className="overview-health-row">
                  <span
                    className={`status-dot ${backendHealthOk ? "status-dot--ok" : "status-dot--error"}`}
                    aria-hidden="true"
                  />
                  <span className="overview-label">Overall</span>
                  <strong className="overview-value">
                    {backendHealthOk ? "ready" : "needs attention"}
                  </strong>
                </div>
                <HealthRow
                  label="Sync backend"
                  ok={backendHealth.sync_backend_reachable}
                />
                <HealthRow
                  label="Bitcoin tip"
                  ok={backendHealth.bitcoin_tip_reachable}
                  value={
                    backendHealth.tip_height === null || backendHealth.tip_height === undefined
                      ? "height n/a"
                      : `height ${formatNumber(backendHealth.tip_height)}`
                  }
                />
                <HealthRow
                  label="Broadcast backend"
                  ok={backendHealth.broadcast_backend_reachable}
                />
                {backendHealth.message && (
                  <p className="overview-health__message">{backendHealth.message}</p>
                )}
              </div>
            ) : backendHealthError ? (
              <span className="overview-empty">Backend health failed: {backendHealthError}</span>
            ) : (
              <span className="overview-empty">No backend health loaded.</span>
            )}
          </div>
        </section>

        {error && <div className="overview-error">Error: {error}</div>}

        <section className="overview-card">
          <div className="overview-card__header">
            <div className="overview-card__icon" aria-hidden="true">
              ▤
            </div>
            <h3 className="overview-card__title">Wallet Status</h3>
          </div>

          <div className="overview-card__body">
            {statusLoading ? (
              <span className="overview-empty">Loading wallet status...</span>
            ) : status ? (
              <div className="overview-metrics">
                <div className="overview-metric">
                  <span className="overview-metric__icon" aria-hidden="true">₿</span>
                  <span className="overview-label">Balance</span>
                  <strong className="overview-value">{formatOptionalSats(status.balance_sat)}</strong>
                </div>

                <div className="overview-metric">
                  <span className="overview-metric__icon" aria-hidden="true">◉</span>
                  <span className="overview-label">UTXOs</span>
                  <strong className="overview-value">{formatNumber(status.utxo_count)}</strong>
                </div>

                <div className="overview-metric">
                  <span className="overview-metric__icon" aria-hidden="true">⛓</span>
                  <span className="overview-label">Block Height</span>
                  <strong className="overview-value">
                    {status.last_block_height === null || status.last_block_height === undefined
                      ? "not synced"
                      : formatNumber(status.last_block_height)}
                  </strong>
                </div>
              </div>
            ) : (
              <span className="overview-empty">No wallet status loaded.</span>
            )}
          </div>
        </section>
      </div>
    </section>
  );
}

type HealthRowProps = {
  label: string;
  ok: boolean;
  value?: string;
};

function HealthRow({ label, ok, value }: HealthRowProps) {
  return (
    <div className="overview-health-row">
      <span
        className={`status-dot ${ok ? "status-dot--ok" : "status-dot--error"}`}
        aria-hidden="true"
      />
      <span className="overview-label">{label}</span>
      <strong className="overview-value">{value ?? (ok ? "ok" : "error")}</strong>
    </div>
  );
}

function formatNumber(value: number): string {
  return Number.isFinite(value) ? value.toLocaleString() : "—";
}
