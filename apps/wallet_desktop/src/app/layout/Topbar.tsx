import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { useWallet } from "../providers/useWallet";
import { syncWallet } from "../../features/wallet/api";
import { getBackendHealth } from "../../features/wallet/api";

type TopbarProps = {
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
};

export function Topbar({ sidebarOpen, onToggleSidebar }: TopbarProps) {
  const location = useLocation();
  const {
    wallets,
    selectedWalletName,
    setSelectedWalletName,
    loadingWallets,
    walletError,
  } = useWallet();

  const [syncing, setSyncing] = useState(false);
  const [syncError, setSyncError] = useState<string | null>(null);
  const [health, setHealth] = useState<import("../../shared/types/dtos").WalletBackendHealthDto | null>(null);

  const pageTitle = getPageTitle(location.pathname);

  const handleSync = async () => {
    if (!selectedWalletName) return;

    setSyncing(true);
    setSyncError(null);

    try {
      await syncWallet(selectedWalletName);
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      setSyncError(message);
    } finally {
      setSyncing(false);
    }
  };

  const fetchHealth = async () => {
    if (!selectedWalletName) return;
    try {
      const result = await getBackendHealth(selectedWalletName);
      setHealth(result);
    } catch (e) {
      console.error("health fetch failed", e);
    }
  };

  useEffect(() => {
    fetchHealth();
  }, [selectedWalletName]);

  return (
    <header className="topbar">
      <div className="topbar-left">
        <button
          type="button"
          className="topbar-toggle"
          onClick={onToggleSidebar}
          aria-label={sidebarOpen ? "Hide sidebar" : "Show sidebar"}
          title={sidebarOpen ? "Hide sidebar" : "Show sidebar"}
        >
          ☰
        </button>
        <div className="topbar-logo" aria-hidden="true">
          D
        </div>
        <span className="topbar-brand topbar-brand--mobile">Rust Descriptor Wallet</span>
      </div>

      <div className="topbar-center">
        <span className="topbar-title">{pageTitle}</span>
      </div>

      <div className="topbar-right">
        {health && (
          <span
            className={`topbar-health ${
              health.sync_backend_reachable && health.broadcast_backend_reachable
                ? "topbar-health--green"
                : health.sync_backend_reachable || health.broadcast_backend_reachable
                ? "topbar-health--yellow"
                : "topbar-health--red"
            }`}
            title={health.message ?? "Backend health"}
          >
            ●
          </span>
        )}
        {loadingWallets ? (
          <span className="topbar-loading">Loading wallets...</span>
        ) : (
          <select
            className="topbar-wallet-select"
            value={selectedWalletName ?? ""}
            onChange={(event) => setSelectedWalletName(event.target.value)}
          >
            {wallets.length === 0 ? (
              <option value="">No wallets found</option>
            ) : null}
            {wallets.map((wallet) => (
              <option key={wallet.name} value={wallet.name}>
                {wallet.name} ({wallet.network})
              </option>
            ))}
          </select>
        )}

        <button
          type="button"
          className="topbar-sync-button"
          onClick={handleSync}
          disabled={!selectedWalletName || syncing}
        >
          {syncing ? "Syncing..." : "Sync"}
        </button>

        {(walletError || syncError) && (
          <span className="topbar-error">{walletError ?? syncError}</span>
        )}
      </div>
    </header>
  );
}

function getPageTitle(pathname: string): string {
  switch (pathname) {
    case "/":
    case "/overview":
      return "Overview";
    case "/utxos":
      return "UTXOs";
    case "/send":
      return "Send";
    case "/transactions":
      return "Transactions";
    default:
      return "Wallet";
  }
}