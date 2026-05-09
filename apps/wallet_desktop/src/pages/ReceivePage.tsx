import { useState } from "react";
import type { WalletReceiveAddressDto } from "../shared/types/dtos";
import { getReceiveAddress } from "../features/receive/api";
import { ReceiveAddressCard } from "../features/receive/components/ReceiveAddressCard";
import { ReceiveEmptyState } from "../features/receive/components/ReceiveEmptyState";
import { useWallet } from "../app/providers/useWallet";

export function ReceivePage() {
  const { selectedWalletName } = useWallet();

  const [address, setAddress] = useState<WalletReceiveAddressDto | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
        />
      ) : (
        <ReceiveEmptyState
          walletName={selectedWalletName ?? ""}
          loading={loading}
          error={error}
          onGenerate={handleGenerate}
        />
      )}
    </section>
  );
}
