pub mod wallet;
pub mod runtime;

use anyhow::Result;
use wallet_api::WalletApi;

use crate::cli::Commands;

pub async fn handle_command(api: &WalletApi, cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Status { name } => {
            runtime::status(api, &name).await?;
        }
        Commands::ListWallets => {
            wallet::list_wallets(api).await?;
        }
        Commands::GetWallet { name } => {
            wallet::get_wallet(api, &name).await?;
        }
        Commands::ImportWallet { file } => {
            wallet::import_wallet(api, file.as_path()).await?;
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
        Commands::Txs { name } => {
            runtime::txs(api, &name).await?;
        }
        Commands::Utxos { name } => {
            runtime::utxos(api, &name).await?;
        }
        Commands::CreatePsbt {
            name,
            to,
            amount,
            fee_rate,
        } => {
            runtime::create_psbt(api, &name, &to, amount, fee_rate).await?;
        }
        Commands::CreatePsbtWithCoinControl {
            name,
            to,
            amount,
            fee_rate,
            include,
            exclude,
            confirmed_only,
        } => {
            runtime::create_psbt_with_coin_control(
                api,
                &name,
                &to,
                amount,
                fee_rate,
                include,
                exclude,
                confirmed_only,
            )
            .await?;
        }
        Commands::SignPsbt { name, psbt_base64 } => {
            runtime::sign_psbt(api, &name, &psbt_base64).await?;
        }
        Commands::PublishPsbt { name, psbt_base64 } => {
            runtime::publish_psbt(api, &name, &psbt_base64).await?;
        }
        Commands::BumpFeePsbt { name, txid, fee_rate } => {
            runtime::bump_fee_psbt(api, &name, &txid, fee_rate).await?;
        }
        Commands::BumpFee { name, txid, fee_rate } => {
            runtime::bump_fee(api, &name, &txid, fee_rate).await?;
        }
        Commands::SendPsbt {
            name,
            to,
            amount,
            fee_rate,
        } => {
            runtime::send_psbt(api, &name, &to, amount, fee_rate).await?;
        }
        Commands::SendPsbtWithCoinControl {
            name,
            to,
            amount,
            fee_rate,
            include,
            exclude,
            confirmed_only,
        } => {
            runtime::send_psbt_with_coin_control(
                api,
                &name,
                &to,
                amount,
                fee_rate,
                include,
                exclude,
                confirmed_only,
            )
            .await?;
        }
        Commands::CpfpPsbt {
            name,
            parent_txid,
            selected_outpoint,
            fee_rate,
        } => {
            runtime::cpfp_psbt(api, &name, &parent_txid, &selected_outpoint, fee_rate).await?;
        }
    }

    Ok(())
}