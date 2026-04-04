use std::fmt;
use crate::WalletCoreError;

/// Strongly-typed amount expressed in satoshis.
///
/// Using a dedicated type prevents accidentally mixing wallet amounts
/// with other numeric values in transaction-building code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AmountSat(pub u64);

impl AmountSat {
    /// Create a validated satoshi amount.
    ///
    /// Zero is rejected because send flow currently expects a strictly
    /// positive recipient amount.
    pub fn new(value: u64) -> Result<Self, WalletCoreError> {
        if value == 0 {
            return Err(WalletCoreError::InvalidAmount);
        }
        Ok(Self(value))
    }

    /// Access the inner raw satoshi value.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<AmountSat> for u64 {
    fn from(value: AmountSat) -> Self {
        value.0
    }
}

impl fmt::Display for AmountSat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Strongly-typed fee rate expressed in satoshis per virtual byte.
///
/// This avoids mixing fee rate values with plain satoshi amounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, )]
pub struct FeeRateSatPerVb(pub u64);

impl FeeRateSatPerVb {
    /// Create a validated fee rate.
    ///
    /// Zero is rejected because the current PSBT flow expects a positive fee rate.
    pub fn new(value: u64) -> Result<Self, WalletCoreError> {
        if value == 0 {
            return Err(WalletCoreError::InvalidFeeRate);
        }
        Ok(Self(value))
    }

    /// Access the inner raw fee-rate value.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<FeeRateSatPerVb> for u64 {
    fn from(value: FeeRateSatPerVb) -> Self {
        value.0
    }
}

impl fmt::Display for FeeRateSatPerVb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} sat/vB", self.0)
    }
}

/// Wallet keychain (derivation path branch)
///
/// External = receiving addresses
/// Internal = change addresses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletKeychain {
    External,
    Internal,
}

impl WalletKeychain {
    pub fn as_str(self) -> &'static str {
        match self {
            WalletKeychain::External => "external",
            WalletKeychain::Internal => "internal",
        }
    }
}

/// Transaction direction relative to the wallet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxDirection {
    Received,
    Sent,
    SelfTransfer,
}

impl TxDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            TxDirection::Received => "received",
            TxDirection::Sent => "sent",
            TxDirection::SelfTransfer => "self",
        }
    }
}