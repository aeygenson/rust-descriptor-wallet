

use crate::model::{DescriptorViewDto, WalletDescriptorInfoDto};
use crate::WalletApiResult;
use wallet_storage::WalletStorage;

/// Build a UI-safe descriptor inspection DTO for a wallet.
///
/// This function is intentionally separate from the runtime wallet-loading path.
/// It must never expose raw private descriptor material, RPC credentials, backend
/// config, or wallet database paths.
pub async fn get_wallet_descriptor_info(
    storage: &WalletStorage,
    wallet_name: &str,
) -> WalletApiResult<WalletDescriptorInfoDto> {
    let wallet = storage.get_wallet_by_name(wallet_name).await?;

    let external = inspect_descriptor(&wallet.external_descriptor);
    let internal = if wallet.internal_descriptor.trim().is_empty() {
        None
    } else {
        Some(inspect_descriptor(&wallet.internal_descriptor))
    };

    let contains_private_data = external.has_private_keys
        || internal
            .as_ref()
            .map(|descriptor| descriptor.has_private_keys)
            .unwrap_or(false);

    Ok(WalletDescriptorInfoDto {
        wallet_name: wallet.name,
        network: wallet.network,
        is_watch_only: wallet.is_watch_only,
        external,
        internal,
        contains_private_data,
    })
}

fn inspect_descriptor(descriptor: &str) -> DescriptorViewDto {
    let descriptor = descriptor.trim();
    let descriptor_redacted = redact_descriptor(descriptor);
    let has_private_keys = contains_private_descriptor_material(descriptor);

    DescriptorViewDto {
        descriptor_redacted,
        script_type: infer_script_type(descriptor),
        has_private_keys,
        has_wildcards: descriptor.contains('*'),
        has_origin_info: descriptor.contains('[') && descriptor.contains(']'),
        is_multisig: is_multisig_descriptor(descriptor),
        threshold: infer_multisig_threshold(descriptor),
        participant_count: infer_multisig_participant_count(descriptor),
        derivation_path: infer_derivation_path(descriptor),
    }
}

fn redact_descriptor(descriptor: &str) -> String {
    descriptor
        .split(|character: char| is_descriptor_separator(character))
        .fold(descriptor.to_string(), |redacted, token| {
            let token = token.trim();
            if token.is_empty() || !is_sensitive_token(token) {
                return redacted;
            }

            redacted.replace(token, &redact_token(token))
        })
}

fn is_descriptor_separator(character: char) -> bool {
    matches!(
        character,
        '(' | ')' | '[' | ']' | ',' | '/' | ':' | '#' | '\n' | '\r' | '\t' | ' '
    )
}

fn contains_private_descriptor_material(descriptor: &str) -> bool {
    descriptor
        .split(|character: char| is_descriptor_separator(character))
        .any(|token| is_sensitive_token(token.trim()))
}

fn is_sensitive_token(token: &str) -> bool {
    let token = token.trim();

    token.starts_with("xprv")
        || token.starts_with("tprv")
        || token.starts_with("yprv")
        || token.starts_with("zprv")
        || token.starts_with("uprv")
        || token.starts_with("vprv")
        || looks_like_wif_private_key(token)
}

fn looks_like_wif_private_key(token: &str) -> bool {
    let length = token.len();

    matches!(token.chars().next(), Some('5' | 'K' | 'L' | 'c'))
        && (51..=52).contains(&length)
        && token.chars().all(is_base58_character)
}

fn is_base58_character(character: char) -> bool {
    matches!(
        character,
        '1'..='9'
            | 'A'..='H'
            | 'J'..='N'
            | 'P'..='Z'
            | 'a'..='k'
            | 'm'..='z'
    )
}

fn redact_token(token: &str) -> String {
    if token.starts_with("xprv")
        || token.starts_with("tprv")
        || token.starts_with("yprv")
        || token.starts_with("zprv")
        || token.starts_with("uprv")
        || token.starts_with("vprv")
    {
        return "<redacted-extended-private-key>".to_string();
    }

    if looks_like_wif_private_key(token) {
        return "<redacted-wif-private-key>".to_string();
    }

    "<redacted-private-material>".to_string()
}

fn infer_script_type(descriptor: &str) -> Option<String> {
    let descriptor = descriptor.trim();

    if descriptor.starts_with("tr(") {
        Some("tr".to_string())
    } else if descriptor.starts_with("wpkh(") {
        Some("wpkh".to_string())
    } else if descriptor.starts_with("sh(wpkh(") {
        Some("sh-wpkh".to_string())
    } else if descriptor.starts_with("wsh(") {
        Some("wsh".to_string())
    } else if descriptor.starts_with("sh(wsh(") {
        Some("sh-wsh".to_string())
    } else if descriptor.starts_with("sh(") {
        Some("sh".to_string())
    } else if descriptor.starts_with("pkh(") {
        Some("pkh".to_string())
    } else if descriptor.starts_with("pk(") {
        Some("pk".to_string())
    } else if descriptor.starts_with("combo(") {
        Some("combo".to_string())
    } else if descriptor.starts_with("addr(") {
        Some("addr".to_string())
    } else if descriptor.starts_with("raw(") {
        Some("raw".to_string())
    } else {
        None
    }
}

fn is_multisig_descriptor(descriptor: &str) -> bool {
    descriptor.contains("multi(") || descriptor.contains("sortedmulti(")
}

fn infer_multisig_threshold(descriptor: &str) -> Option<u32> {
    let argument_text = multisig_argument_text(descriptor)?;
    let threshold = argument_text.split(',').next()?.trim();

    threshold.parse::<u32>().ok()
}

fn infer_multisig_participant_count(descriptor: &str) -> Option<u32> {
    let argument_text = multisig_argument_text(descriptor)?;
    let argument_count = argument_text.split(',').count();

    argument_count.checked_sub(1).map(|count| count as u32)
}

fn multisig_argument_text(descriptor: &str) -> Option<&str> {
    let marker = if let Some(index) = descriptor.find("sortedmulti(") {
        (index, "sortedmulti(".len())
    } else if let Some(index) = descriptor.find("multi(") {
        (index, "multi(".len())
    } else {
        return None;
    };

    let start = marker.0 + marker.1;
    let rest = &descriptor[start..];
    let end = rest.find(')')?;

    Some(&rest[..end])
}

fn infer_derivation_path(descriptor: &str) -> Option<String> {
    let origin_start = descriptor.find('[')?;
    let origin_end = descriptor[origin_start..].find(']')? + origin_start;
    let origin = &descriptor[origin_start + 1..origin_end];
    let mut parts = origin.split('/');

    parts.next()?;

    let path = parts.collect::<Vec<_>>().join("/");
    if path.is_empty() {
        None
    } else {
        Some(format!("/{path}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_extended_private_keys() {
        let descriptor = "wpkh([abcd1234/84h/1h/0h]tprv8ZgxMBicQKsPdExamplePrivateKeyMaterial/0/*)";
        let view = inspect_descriptor(descriptor);

        assert!(view.has_private_keys);
        assert!(view
            .descriptor_redacted
            .contains("<redacted-extended-private-key>"));
        assert!(!view.descriptor_redacted.contains("tprv"));
        assert!(!view.descriptor_redacted.contains("tprv8ZgxMBicQKsPdExamplePrivateKeyMaterial"));
    }

    #[test]
    fn infers_basic_wpkh_metadata() {
        let descriptor = "wpkh([abcd1234/84h/1h/0h]xpub661MyMwAqRbcExample/0/*)";
        let view = inspect_descriptor(descriptor);

        assert_eq!(view.script_type.as_deref(), Some("wpkh"));
        assert!(view.has_wildcards);
        assert!(view.has_origin_info);
        assert_eq!(view.derivation_path.as_deref(), Some("/84h/1h/0h"));
        assert!(!view.has_private_keys);
    }

    #[test]
    fn infers_multisig_metadata() {
        let descriptor = "wsh(sortedmulti(2,xpub1/0/*,xpub2/0/*,xpub3/0/*))";
        let view = inspect_descriptor(descriptor);

        assert!(view.is_multisig);
        assert_eq!(view.threshold, Some(2));
        assert_eq!(view.participant_count, Some(3));
    }
}
