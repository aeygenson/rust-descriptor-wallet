

import { createContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { listWallets } from "../../features/wallet/api";
import type { WalletSummaryDto } from "../../shared/types/dtos";

type WalletContextValue = {
  wallets: WalletSummaryDto[];
  selectedWalletName: string;
  selectedWallet: WalletSummaryDto | null;
  loadingWallets: boolean;
  walletError: string | null;
  setSelectedWalletName: (walletName: string) => void;
  refreshWallets: () => Promise<void>;
};

export const WalletContext = createContext<WalletContextValue | null>(null);

type WalletProviderProps = {
  children: ReactNode;
};

export function WalletProvider({ children }: WalletProviderProps) {
  const [wallets, setWallets] = useState<WalletSummaryDto[]>([]);
  const [selectedWalletName, setSelectedWalletName] = useState("");
  const [loadingWallets, setLoadingWallets] = useState(false);
  const [walletError, setWalletError] = useState<string | null>(null);

  const refreshWallets = async () => {
    setLoadingWallets(true);
    setWalletError(null);

    try {
      const loadedWallets = await listWallets();
      setWallets(loadedWallets);

      if (loadedWallets.length === 0) {
        setSelectedWalletName("");
        return;
      }

      setSelectedWalletName((current) => {
        const currentStillExists = loadedWallets.some((wallet) => wallet.name === current);
        return currentStillExists ? current : loadedWallets[0].name;
      });
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      setWalletError(message);
      setWallets([]);
      setSelectedWalletName("");
    } finally {
      setLoadingWallets(false);
    }
  };

  useEffect(() => {
    void refreshWallets();
  }, []);

  const selectedWallet = useMemo(
    () => wallets.find((wallet) => wallet.name === selectedWalletName) ?? null,
    [wallets, selectedWalletName],
  );

  const value = useMemo<WalletContextValue>(
    () => ({
      wallets,
      selectedWalletName,
      selectedWallet,
      loadingWallets,
      walletError,
      setSelectedWalletName,
      refreshWallets,
    }),
    [wallets, selectedWalletName, selectedWallet, loadingWallets, walletError],
  );

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}