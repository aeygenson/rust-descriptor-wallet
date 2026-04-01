

pub mod wallet;
pub mod runtime;

use anyhow::Result;
use wallet_api::WalletApi;

use crate::cli::Commands;

pub async fn handle_command(api: &WalletApi, cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Status => {
            let status = api.status().await?;
            println!("{status}");
        }
        Commands::ListWallets => {
            wallet::list_wallets(api).await?;
        }
        Commands::GetWallet { name } => {
            wallet::get_wallet(api, &name).await?;
        }
        Commands::ImportWallet { file } => {
            wallet::import_wallet(api, &file).await?;
        }
        Commands::DeleteWallet { name } => {
            wallet::delete_wallet(api, &name).await?;
        }
        Commands::Address { name } => {
            runtime::address(api, &name).await?;
        }
        Commands::Sync { name } => {
            runtime::sync(api, &name).await?;
        }
        Commands::Balance { name } => {
            runtime::balance(api, &name).await?;
        }
    }

    Ok(())
}