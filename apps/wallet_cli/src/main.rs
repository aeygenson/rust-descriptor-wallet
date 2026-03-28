use anyhow::Result;
use clap::{Parser, Subcommand};
use wallet_api::WalletApi;

#[derive(Debug, Parser)]
#[command(name = "wallet")]
#[command(about = "Rust Descriptor Wallet CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Status,
    LoadMarker {
        key: String,
    },
    ListWallets,
    GetWallet {
        name: String,
    },
    ImportWallet {
        #[arg(long)]
        file: String,
    },
    DeleteWallet {
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let api = WalletApi::new().await?;
    
    match cli.command {
        Commands::Status => {
            let status = api.status().await?;
            println!("{status}");
        }
        Commands::LoadMarker { key } => {
            let value = api.load_marker(&key).await?;
            println!("{value}");
        }
        Commands::ListWallets => {
            let wallets = api.list_wallets().await?;
            
            if wallets.is_empty() {
                println!("No wallets found.");
            } else {
                for wallet in wallets {
                    println!(
                        "name={} network={} watch_only={}",
                        wallet.name, wallet.network, wallet.is_watch_only
                    );
                }
            }
        }
        Commands::GetWallet { name } => {
            let wallet = api.get_wallet(&name).await?;
            println!("name={}", wallet.name);
            println!("network={}", wallet.network);
            println!("watch_only={}", wallet.is_watch_only);
            println!("esplora_url={}", wallet.esplora_url);
            println!("external_descriptor={}", wallet.external_descriptor);
            println!("internal_descriptor={}", wallet.internal_descriptor);
        }
        Commands::ImportWallet { file } => {
            api.import_wallet(&file).await?;
            println!("Imported wallet from {file}");
        }
        Commands::DeleteWallet { name } => {
            api.delete_wallet(&name).await?;
            println!("Deleted wallet {name}");
        }
    }
    
    Ok(())
}