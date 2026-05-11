import { useCallback, useEffect, useState } from "react";
import type { WalletReceiveAddressHistoryDto } from "../shared/types/dtos";
import {
  clearReceiveAddressLabel,
  getReceiveAddress,
  labelReceiveAddress,
  listReceiveAddresses,
} from "../features/receive/api";
import { ReceiveAddressCard } from "../features/receive/components/ReceiveAddressCard";
import { ReceiveEmptyState } from "../features/receive/components/ReceiveEmptyState";
import { ReceiveAddressHistoryList } from "../features/receive/components/ReceiveAddressHistoryList";
import { useWallet } from "../app/providers/useWallet";

export function ReceivePage() {
  const { selectedWalletName } = useWallet();

  const [address, setAddress] = useState<WalletReceiveAddressHistoryDto | null>(null);
  const [history, setHistory] = useState<WalletReceiveAddressHistoryDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadHistory = useCallback(async () => {
    if (!selectedWalletName) {
      setHistory([]);
      return;
    }

    try {
      const result = await listReceiveAddresses(selectedWalletName);
      setHistory(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [selectedWalletName]);

  useEffect(() => {
    void loadHistory();
  }, [loadHistory]);

  const handleGenerate = async () => {
    if (!selectedWalletName) {
      setError("No wallet selected");
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const result = await getReceiveAddress(selectedWalletName);
      setAddress(result);
      await loadHistory();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleSaveLabel = async (label: string) => {
    if (!selectedWalletName || !address) {
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const updated = await labelReceiveAddress(
        selectedWalletName,
        address.address,
        label,
      );
      setAddress(updated);
      await loadHistory();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleClearLabel = async () => {
    if (!selectedWalletName || !address) {
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const updated = await clearReceiveAddressLabel(
        selectedWalletName,
        address.address,
      );
      setAddress(updated);
      await loadHistory();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <section className="receive-page">
      <header className="receive-page__header">
        <div>
          <h1 className="receive-page__title">Receive</h1>

          <p className="receive-page__subtitle">
            Generate wallet-controlled receive addresses for the active wallet.
          </p>
        </div>

        <div className="receive-wallet-pill">
          {selectedWalletName ?? "No wallet selected"}
        </div>
      </header>

      {address ? (
        <ReceiveAddressCard
          walletName={selectedWalletName ?? ""}
          address={address}
          loading={loading}
          onRefresh={handleGenerate}
          onCopy={() => undefined}
          onSaveLabel={handleSaveLabel}
          onClearLabel={handleClearLabel}
        />
      ) : (
        <ReceiveEmptyState
          walletName={selectedWalletName ?? ""}
          loading={loading}
          error={error}
          onGenerate={handleGenerate}
        />
      )}

      <section className="receive-page__history">
        <div className="receive-page__section-header">
          <h2 className="receive-page__section-title">Receive history</h2>
          <p className="receive-page__section-subtitle">
            Recently generated addresses stored for this wallet.
          </p>
        </div>

        <ReceiveAddressHistoryList
          walletName={selectedWalletName ?? ""}
          addresses={history}
          loading={loading}
          onSelect={setAddress}
        />
      </section>
    </section>
  );
}
