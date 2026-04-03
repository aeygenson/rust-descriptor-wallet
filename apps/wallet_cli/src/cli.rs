use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "wallet")]
#[command(about = "Rust Descriptor Wallet CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Status {
        #[arg(long)]
        name: String,
    },
    ListWallets,
    GetWallet {
        #[arg(long)]
        name: String,
    },
    ImportWallet {
        #[arg(long)]
        file: String,
    },
    DeleteWallet {
        #[arg(long)]
        name: String,
    },
    Address {
        #[arg(long)]
        name: String,
    },
    Sync {
        #[arg(long)]
        name: String,
    },
    Balance {
        #[arg(long)]
        name: String,
    },
    Txs {
        #[arg(long)]
        name: String,
    },
    Utxos {
        #[arg(long)]
        name: String,
    },
}