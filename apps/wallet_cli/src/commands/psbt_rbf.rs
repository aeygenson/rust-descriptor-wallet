use anyhow::Result;

use wallet_api::model::{
    BumpFeeRequestDto, CpfpRequestDto, TxBroadcastResultDto, WalletCpfpPsbtDto,
    WalletPsbtDto,
};
use wallet_api::{service, WalletApi};

fn print_optional_rbf(replaceable: Option<bool>) {
    if let Some(replaceable) = replaceable {
        println!("replaceable={}", replaceable);
    }
}

fn print_broadcast_success(title: &str, published: &TxBroadcastResultDto) {
    println!("{}", title);
    println!("txid={}", published.txid);
    print_optional_rbf(published.replaceable);
}

fn print_wallet_psbt(title: &str, psbt: &WalletPsbtDto) {
    println!("{}", title);
    println!("psbt_base64:\n{}", psbt.psbt_base64);
    println!("fee_sat={}", psbt.fee_sat);
    println!("fee_rate_sat_per_vb={}", psbt.fee_rate_sat_per_vb);
    println!("input_count={}", psbt.input_count);
    println!("replaceable={}", psbt.replaceable);

    if let Some(change_amount_sat) = psbt.change_amount_sat {
        println!("change_amount_sat={}", change_amount_sat);
    }

    if psbt.replacement.is_some() {
        println!("replacement=true");
    }
}

fn print_cpfp_psbt(title: &str, psbt: &WalletCpfpPsbtDto) {
    println!("{}", title);
    println!("txid={}", psbt.txid);
    println!("psbt_base64:\n{}", psbt.psbt_base64);
    println!("fee_sat={}", psbt.fee_sat);
    println!("fee_rate_sat_per_vb={}", psbt.fee_rate_sat_per_vb);
    println!("input_value_sat={}", psbt.input_value_sat);
    println!("child_output_value_sat={}", psbt.child_output_value_sat);
    println!("estimated_vsize={}", psbt.estimated_vsize);
    println!("replaceable={}", psbt.replaceable);
    println!("parent_txid={}", psbt.parent_txid);
    println!("selected_outpoint={}", psbt.selected_outpoint);
}

pub async fn bump_fee_psbt(
    api: &WalletApi,
    name: &str,
    txid: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    let psbt = service::psbt::bump_fee_psbt(
        &api.storage,
        BumpFeeRequestDto {
            name: name.to_string(),
            txid: txid.to_string(),
            fee_rate_sat_per_vb,
        },
    )
    .await?;

    print_wallet_psbt("RBF PSBT created:", &psbt);
    Ok(())
}

pub async fn bump_fee(
    api: &WalletApi,
    name: &str,
    txid: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    let published = service::psbt::bump_fee(
        &api.storage,
        BumpFeeRequestDto {
            name: name.to_string(),
            txid: txid.to_string(),
            fee_rate_sat_per_vb,
        },
    )
    .await?;

    print_broadcast_success("RBF transaction broadcast:", &published);
    Ok(())
}

pub async fn cpfp_psbt(
    api: &WalletApi,
    name: &str,
    parent_txid: &str,
    selected_outpoint: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    let psbt = service::psbt::cpfp_psbt(
        &api.storage,
        CpfpRequestDto {
            name: name.to_string(),
            parent_txid: parent_txid.to_string(),
            selected_outpoint: selected_outpoint.to_string(),
            fee_rate_sat_per_vb,
        },
    )
    .await?;

    print_cpfp_psbt("CPFP PSBT created:", &psbt);
    Ok(())
}

pub async fn cpfp(
    api: &WalletApi,
    name: &str,
    parent_txid: &str,
    selected_outpoint: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    let published = service::psbt::cpfp(
        &api.storage,
        CpfpRequestDto {
            name: name.to_string(),
            parent_txid: parent_txid.to_string(),
            selected_outpoint: selected_outpoint.to_string(),
            fee_rate_sat_per_vb,
        },
    )
    .await?;

    print_broadcast_success("CPFP transaction broadcast:", &published);
    Ok(())
}