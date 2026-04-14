mod utxos;
mod txs;
mod psbt_create;
mod lifecycle;
pub mod psbt_sign;
pub mod psbt_publish;
pub mod psbt_common;
pub mod psbt_rbf;
pub mod psbt_cpfp;
pub mod psbt_coin_control;

use bdk_file_store::Store;
use bdk_wallet::{ChangeSet, PersistedWallet, Wallet};

/// Magic bytes used by `bdk_file_store` to identify our database format.
///
/// Changing this will make existing databases incompatible.
const MAGIC_BYTES: &[u8] = b"rust-descriptor-wallet-v1";

#[derive(Debug)]
pub struct WalletService {
    /// BDK persisted wallet wrapper.
    wallet: PersistedWallet<Store<ChangeSet>>,

    /// File-backed store used by the persisted wallet.
    db: Store<ChangeSet>,

    /// Indicates whether this wallet is watch-only (cannot sign transactions).
    is_watch_only: bool,
}