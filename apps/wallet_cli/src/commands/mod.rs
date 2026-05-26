pub mod inspect;
pub mod psbt;
pub mod wallet;
pub mod psbt_create;
pub mod psbt_broadcast;
pub mod psbt_rbf;

use anyhow::Result;
use wallet_api::WalletApi;

use crate::cli::Commands;

pub async fn handle_command(api: &WalletApi, cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Status { name } => {
            wallet::status(api, &name).await?;
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
        Commands::Address { name, qr_svg } => {
            wallet::address(api, &name, qr_svg).await?;
        }
        Commands::ReceiveAddresses { name, qr_svg } => {
            wallet::list_receive_addresses(api, &name, qr_svg).await?;
        }
        Commands::LabelReceiveAddress {
            name,
            address,
            label,
        } => {
            wallet::label_receive_address(api, &name, &address, &label).await?;
        }
        Commands::ClearReceiveAddressLabel { name, address } => {
            wallet::clear_receive_address_label(api, &name, &address).await?;
        }
        Commands::AddressBookAdd {
            name,
            label,
            address,
            notes,
        } => {
            wallet::create_address_book_entry(
                api,
                &name,
                &label,
                &address,
                notes,
            )
            .await?;
        }
        Commands::AddressBookList { name } => {
            wallet::list_address_book_entries(api, &name).await?;
        }
        Commands::AddressBookGet { name, address } => {
            wallet::get_address_book_entry(api, &name, &address).await?;
        }
        Commands::AddressBookDelete { name, address } => {
            wallet::delete_address_book_entry(api, &name, &address).await?;
        }
        Commands::LockUtxo {
            name,
            outpoint,
            reason,
        } => {
            wallet::lock_utxo(api, &name, &outpoint, reason).await?;
        }
        Commands::UnlockUtxo { name, outpoint } => {
            wallet::unlock_utxo(api, &name, &outpoint).await?;
        }
        Commands::LockedUtxos { name } => {
            wallet::list_locked_utxos(api, &name).await?;
        }
        Commands::Sync { name } => {
            wallet::sync_wallet(api, &name).await?;
        }
        Commands::Health { name } => {
            wallet::backend_health(api, &name).await?;
        }
        Commands::Balance { name } => {
            wallet::balance(api, &name).await?;
        }
        Commands::Txs { name } => {
            inspect::txs(api, &name).await?;
        }
        Commands::Utxos { name } => {
            inspect::utxos(api, &name).await?;
        }
        Commands::CreatePsbt {
            name,
            to,
            amount,
            fee_rate,
            replaceable,
            confirmed_only,
        } => {
            psbt::create_psbt_with_options(
                api,
                &name,
                &to,
                amount,
                fee_rate,
                replaceable,
                confirmed_only,
            )
            .await?;
        }
        Commands::CreatePsbtWithCoinControl {
            name,
            to,
            amount,
            fee_rate,
            replaceable,
            include,
            exclude,
            confirmed_only,
            selection_mode,
        } => {
            psbt::create_psbt_with_coin_control_and_options(
                api,
                &name,
                &to,
                amount,
                fee_rate,
                replaceable,
                include,
                exclude,
                confirmed_only,
                selection_mode,
            )
            .await?;
        }
        Commands::CreateSendMaxPsbt {
            name,
            to,
            fee_rate,
            replaceable,
        } => {
            psbt::create_send_max_psbt_with_options(api, &name, &to, fee_rate, replaceable)
                .await?;
        }
        Commands::CreateSendMaxPsbtWithCoinControl {
            name,
            to,
            fee_rate,
            replaceable,
            include,
            exclude,
            confirmed_only,
            selection_mode,
        } => {
            psbt::create_send_max_psbt_with_coin_control_and_options(
                api,
                &name,
                &to,
                fee_rate,
                replaceable,
                include,
                exclude,
                confirmed_only,
                selection_mode,
            )
            .await?;
        }
        Commands::SignPsbt { name, psbt_base64 } => {
            psbt::sign_psbt(api, &name, &psbt_base64).await?;
        }
        Commands::PublishPsbt { name, psbt_base64 } => {
            psbt::publish_psbt(api, &name, &psbt_base64).await?;
        }
        Commands::BumpFeePsbt {
            name,
            txid,
            fee_rate,
        } => {
            psbt::bump_fee_psbt(api, &name, &txid, fee_rate).await?;
        }
        Commands::BumpFee {
            name,
            txid,
            fee_rate,
        } => {
            psbt::bump_fee(api, &name, &txid, fee_rate).await?;
        }
        Commands::SendPsbt {
            name,
            to,
            amount,
            fee_rate,
            replaceable,
            confirmed_only,
        } => {
            psbt::send_psbt_with_options(
                api,
                &name,
                &to,
                amount,
                fee_rate,
                replaceable,
                confirmed_only,
            )
            .await?;
        }
        Commands::SendPsbtWithCoinControl {
            name,
            to,
            amount,
            fee_rate,
            replaceable,
            include,
            exclude,
            confirmed_only,
            selection_mode,
        } => {
            psbt::send_psbt_with_coin_control_and_options(
                api,
                &name,
                &to,
                amount,
                fee_rate,
                replaceable,
                include,
                exclude,
                confirmed_only,
                selection_mode,
            )
            .await?;
        }
        Commands::SendMaxPsbt {
            name,
            to,
            fee_rate,
            replaceable,
        } => {
            psbt::send_max_psbt_with_options(api, &name, &to, fee_rate, replaceable).await?;
        }
        Commands::SendMaxPsbtWithCoinControl {
            name,
            to,
            fee_rate,
            replaceable,
            include,
            exclude,
            confirmed_only,
            selection_mode,
        } => {
            psbt::send_max_psbt_with_coin_control_and_options(
                api,
                &name,
                &to,
                fee_rate,
                replaceable,
                include,
                exclude,
                confirmed_only,
                selection_mode,
            )
            .await?;
        }
        Commands::SweepPsbt {
            name,
            to,
            fee_rate,
            replaceable,
            include,
            exclude,
            confirmed_only,
            selection_mode,
        } => {
            psbt::create_sweep_psbt_with_options(
                api,
                &name,
                &to,
                fee_rate,
                replaceable,
                include,
                exclude,
                confirmed_only,
                selection_mode,
            )
            .await?;
        }
        Commands::CreateConsolidationPsbt {
            name,
            fee_rate,
            replaceable,
            include,
            exclude,
            confirmed_only,
            max_input_count,
            min_input_count,
            min_utxo_value_sat,
            max_utxo_value_sat,
            max_fee_pct_of_input_value,
            strategy,
            selection_mode,
        } => {
            psbt::create_consolidation_psbt_with_options(
                api,
                &name,
                fee_rate,
                replaceable,
                include,
                exclude,
                confirmed_only,
                max_input_count,
                min_input_count,
                min_utxo_value_sat,
                max_utxo_value_sat,
                max_fee_pct_of_input_value,
                strategy,
                selection_mode,
            )
            .await?;
        }
        Commands::Sweep {
            name,
            to,
            fee_rate,
            replaceable,
            include,
            exclude,
            confirmed_only,
            selection_mode,
        } => {
            psbt::sweep_psbt_with_options(
                api,
                &name,
                &to,
                fee_rate,
                replaceable,
                include,
                exclude,
                confirmed_only,
                selection_mode,
            )
            .await?;
        }
        Commands::Consolidate {
            name,
            fee_rate,
            replaceable,
            include,
            exclude,
            confirmed_only,
            max_input_count,
            min_input_count,
            min_utxo_value_sat,
            max_utxo_value_sat,
            max_fee_pct_of_input_value,
            strategy,
            selection_mode,
        } => {
            psbt::consolidate_psbt_with_options(
                api,
                &name,
                fee_rate,
                replaceable,
                include,
                exclude,
                confirmed_only,
                max_input_count,
                min_input_count,
                min_utxo_value_sat,
                max_utxo_value_sat,
                max_fee_pct_of_input_value,
                strategy,
                selection_mode,
            )
            .await?;
        }
        Commands::CpfpPsbt {
            name,
            parent_txid,
            selected_outpoint,
            fee_rate,
        } => {
            psbt::cpfp_psbt(api, &name, &parent_txid, &selected_outpoint, fee_rate).await?;
        }
        Commands::Cpfp {
            name,
            parent_txid,
            selected_outpoint,
            fee_rate,
        } => {
            psbt::cpfp(api, &name, &parent_txid, &selected_outpoint, fee_rate).await?;
        }
    }

    Ok(())
}
