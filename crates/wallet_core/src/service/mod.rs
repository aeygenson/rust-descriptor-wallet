pub mod common_outpoint;
pub mod common_selection;
pub mod common_tx;
mod lifecycle;
pub mod psbt_coin_control;
pub mod psbt_coin_selector;
pub mod psbt_consolidation;
pub mod psbt_cpfp;
mod psbt_create;
pub mod psbt_publish;
pub mod psbt_rbf;
pub mod psbt_sign;
mod test_support;
mod txs;
mod utxos;

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
