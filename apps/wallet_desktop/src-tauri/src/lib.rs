mod app;
mod commands;
mod integration;

use wallet_api::WalletApi;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let api =
        tauri::async_runtime::block_on(WalletApi::new()).expect("failed to initialize WalletApi");

    tauri::Builder::default()
        .manage(api)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::wallet::get_app_info,
            commands::wallet::list_wallets,
            commands::wallet::get_wallet_status,
            commands::wallet::sync_wallet,
            commands::wallet::backend_health,
            commands::wallet::get_receive_address,
            commands::wallet::list_receive_addresses,
            commands::wallet::label_receive_address,
            commands::wallet::clear_receive_address_label,
            commands::utxos::list_utxos,
            commands::transactions::list_transactions,
            commands::send::create_psbt,
            commands::send::create_psbt_with_coin_control,
            commands::send::create_send_max_psbt,
            commands::send::create_send_max_psbt_with_coin_control,
            commands::send::create_sweep_psbt,
            commands::send::create_consolidation_psbt,
            commands::send::sign_psbt,
            commands::send::publish_psbt,
            commands::send::send_psbt,
            commands::send::send_psbt_with_coin_control,
            commands::send::send_max_psbt,
            commands::send::send_max_psbt_with_coin_control,
            commands::send::send_sweep_psbt,
            commands::send::consolidate_psbt,
            commands::send::bump_fee_psbt,
            commands::send::bump_fee,
            commands::send::cpfp_psbt,
            commands::send::cpfp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
