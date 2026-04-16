use std::str::FromStr;

use anyhow::Result;

/// Parse a regtest address and require the regtest network.
pub fn parse_regtest_address(
    s: &str,
) -> Result<bitcoin::Address<bitcoin::address::NetworkChecked>> {
    Ok(
        s.parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()?
            .require_network(bitcoin::Network::Regtest)?,
    )
}

/// Parse a transaction id from its string form.
pub fn parse_txid(s: &str) -> Result<bitcoin::Txid> {
    Ok(s.parse()?)
}

/// Return the txid portion of an outpoint string in the form `<txid>:<vout>`.
pub fn outpoint_txid(outpoint: &str) -> &str {
    outpoint.split(':').next().unwrap_or("")
}

/// Decode a PSBT string and return the input outpoints used by the unsigned tx.
pub fn decode_psbt_inputs(psbt_base64: &str) -> Result<Vec<String>> {
    let psbt = bitcoin::psbt::Psbt::from_str(psbt_base64)?;

    Ok(psbt
        .unsigned_tx
        .input
        .iter()
        .map(|input| input.previous_output.to_string())
        .collect())
}
