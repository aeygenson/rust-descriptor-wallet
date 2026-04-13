use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "wallet")]
#[command(about = "Rust Descriptor Wallet CLI")]
pub struct Cli {
    #[command(subcommand)]
    /// Wallet command to execute.
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show wallet status summary.
    Status {
        #[arg(long)]
        /// Wallet name.
        name: String,
    },
    /// List all registered wallets.
    ListWallets,
    /// Show stored wallet configuration details.
    GetWallet {
        #[arg(long)]
        /// Wallet name.
        name: String,
    },
    /// Import a wallet definition from a JSON file.
    ImportWallet {
        #[arg(long)]
        /// Path to the wallet JSON file.
        file: PathBuf,
    },
    /// Delete a wallet from the registry.
    DeleteWallet {
        #[arg(long)]
        /// Wallet name.
        name: String,
    },
    /// Generate the next receive address.
    Address {
        #[arg(long)]
        /// Wallet name.
        name: String,
    },
    /// Synchronize wallet state with the configured backend.
    Sync {
        #[arg(long)]
        /// Wallet name.
        name: String,
    },
    /// Show wallet balance.
    Balance {
        #[arg(long)]
        /// Wallet name.
        name: String,
    },
    /// List wallet transactions.
    Txs {
        #[arg(long)]
        /// Wallet name.
        name: String,
    },
    /// List wallet UTXOs.
    Utxos {
        #[arg(long)]
        /// Wallet name.
        name: String,
    },
    /// Create a PSBT without signing or broadcasting it.
    CreatePsbt {
        #[arg(long)]
        /// Wallet name.
        name: String,

        #[arg(long)]
        /// Destination address.
        to: String,

        #[arg(long)]
        /// Amount in satoshis.
        amount: u64,

        #[arg(long = "fee-rate")]
        /// Fee rate in sat/vB.
        fee_rate: u64,
    },
    /// Sign an existing PSBT.
    SignPsbt {
        #[arg(long)]
        /// Wallet name.
        name: String,

        #[arg(long = "psbt-base64")]
        /// PSBT encoded as base64.
        psbt_base64: String,
    },
    /// Broadcast an already finalized PSBT.
    PublishPsbt {
        #[arg(long)]
        /// Wallet name.
        name: String,

        #[arg(long = "psbt-base64")]
        /// PSBT encoded as base64.
        psbt_base64: String,
    },
    /// Build a replacement PSBT for an existing RBF transaction.
    BumpFeePsbt {
        #[arg(long)]
        /// Wallet name.
        name: String,

        #[arg(long, alias = "txid")]
        /// Transaction id (RBF parent) to bump.
        txid: String,

        #[arg(long = "fee-rate")]
        /// Fee rate in sat/vB.
        fee_rate: u64,
    },
    /// Build, sign, and broadcast a replacement transaction.
    BumpFee {
        #[arg(long)]
        /// Wallet name.
        name: String,

        #[arg(long, alias = "txid")]
        /// Transaction id (RBF parent) to bump.
        txid: String,

        #[arg(long = "fee-rate")]
        /// Fee rate in sat/vB.
        fee_rate: u64,
    },
    /// Create, sign, and broadcast a transaction in one step.
    SendPsbt {
        #[arg(long)]
        /// Wallet name.
        name: String,

        #[arg(long)]
        /// Destination address.
        to: String,

        #[arg(long)]
        /// Amount in satoshis.
        amount: u64,

        #[arg(long = "fee-rate")]
        /// Fee rate in sat/vB.
        fee_rate: u64,
    },
    /// Create a CPFP (Child-Pays-For-Parent) PSBT to accelerate a stuck transaction.
    CpfpPsbt {
        #[arg(long)]
        /// Wallet name.
        name: String,

        #[arg(long)]
        /// Parent transaction id to accelerate.
        parent_txid: String,

        #[arg(long = "outpoint")]
        /// Selected outpoint in the form <txid>:<vout> to spend for CPFP.
        selected_outpoint: String,

        #[arg(long = "fee-rate")]
        /// Target fee rate in sat/vB for the package.
        fee_rate: u64,
    },
}