use std::fmt;

use bdk_wallet::bitcoin::FeeRate;

use crate::WalletCoreError;

/// Strongly-typed amount expressed in satoshis.
///
/// Using a dedicated type prevents accidentally mixing wallet amounts
/// with other numeric values in transaction-building code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
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

impl From<u64> for AmountSat {
    fn from(value: u64) -> Self {
        Self(value)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
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

    /// Returns true if the fee rate is zero.
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Ensures the fee rate is non-zero, returning an error otherwise.
    pub fn ensure_non_zero(self) -> Result<Self, WalletCoreError> {
        if self.is_zero() {
            Err(WalletCoreError::InvalidFeeRate)
        } else {
            Ok(self)
        }
    }

    /// Convert into BDK's fee-rate type after validating domain constraints.
    pub fn try_into_bdk(self) -> Result<FeeRate, WalletCoreError> {
        let value = self.ensure_non_zero()?;
        FeeRate::from_sat_per_vb(value.as_u64()).ok_or(WalletCoreError::InvalidFeeRate)
    }
}

impl From<FeeRateSatPerVb> for u64 {
    fn from(value: FeeRateSatPerVb) -> Self {
        value.0
    }
}

impl From<u64> for FeeRateSatPerVb {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<FeeRateSatPerVb> for FeeRate {
    type Error = WalletCoreError;

    fn try_from(value: FeeRateSatPerVb) -> Result<Self, Self::Error> {
        value.try_into_bdk()
    }
}

impl From<FeeRate> for FeeRateSatPerVb {
    fn from(value: FeeRate) -> Self {
        Self(value.to_sat_per_vb_ceil())
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WalletKeychain {
    #[default]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TxDirection {
    Received,
    Sent,
    #[default]
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