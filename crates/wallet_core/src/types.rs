use bdk_wallet::bitcoin::psbt::Psbt;
use bdk_wallet::bitcoin::{FeeRate, OutPoint, Txid};
use std::fmt;
use std::str::FromStr;

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
    /// Zero fee rate constant for placeholder/minimal model construction.
    pub const ZERO: Self = Self(0);
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

/// Strongly-typed wallet outpoint.
///
/// This avoids passing raw strings through core wallet logic once an outpoint
/// has been parsed and validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalletOutPoint(pub OutPoint);

impl WalletOutPoint {
    /// Create a validated wallet outpoint from a Bitcoin outpoint.
    pub fn new(value: OutPoint) -> Self {
        Self(value)
    }

    /// Parse a validated wallet outpoint from its standard `txid:vout` string form.
    pub fn parse(s: &str) -> Result<Self, WalletCoreError> {
        s.parse()
    }

    /// Borrow the inner Bitcoin outpoint by value-friendly name.
    pub fn inner(&self) -> OutPoint {
        self.0
    }

    /// Consume the wrapper and return the inner Bitcoin outpoint.
    pub fn into_inner(self) -> OutPoint {
        self.0
    }
}

impl From<OutPoint> for WalletOutPoint {
    fn from(value: OutPoint) -> Self {
        Self(value)
    }
}

impl From<&OutPoint> for WalletOutPoint {
    fn from(value: &OutPoint) -> Self {
        Self(*value)
    }
}

impl From<WalletOutPoint> for OutPoint {
    fn from(value: WalletOutPoint) -> Self {
        value.0
    }
}

impl AsRef<OutPoint> for WalletOutPoint {
    fn as_ref(&self) -> &OutPoint {
        &self.0
    }
}

impl FromStr for WalletOutPoint {
    type Err = WalletCoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        OutPoint::from_str(s)
            .map(Self)
            .map_err(|_| WalletCoreError::InvalidOutpoint(s.to_string()))
    }
}

impl TryFrom<&str> for WalletOutPoint {
    type Error = WalletCoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for WalletOutPoint {
    type Error = WalletCoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl fmt::Display for WalletOutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Strongly-typed wallet transaction id.
///
/// This avoids mixing txids with arbitrary strings inside wallet-core domain
/// logic while still allowing clean conversion at API boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalletTxid(pub Txid);

impl WalletTxid {
    /// Create a validated wallet txid from a Bitcoin txid.
    pub fn new(value: Txid) -> Self {
        Self(value)
    }

    /// Parse a validated wallet txid from its standard hex string form.
    pub fn parse(s: &str) -> Result<Self, WalletCoreError> {
        s.parse()
    }

    /// Return the inner Bitcoin txid by value.
    pub fn inner(&self) -> Txid {
        self.0
    }

    /// Consume the wrapper and return the inner Bitcoin txid.
    pub fn into_inner(self) -> Txid {
        self.0
    }
}

impl From<Txid> for WalletTxid {
    fn from(value: Txid) -> Self {
        Self(value)
    }
}

impl From<&Txid> for WalletTxid {
    fn from(value: &Txid) -> Self {
        Self(*value)
    }
}

impl From<WalletTxid> for Txid {
    fn from(value: WalletTxid) -> Self {
        value.0
    }
}

impl AsRef<Txid> for WalletTxid {
    fn as_ref(&self) -> &Txid {
        &self.0
    }
}

impl FromStr for WalletTxid {
    type Err = WalletCoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Txid::from_str(s)
            .map(Self)
            .map_err(|_| WalletCoreError::InvalidTxid(s.to_string()))
    }
}

impl TryFrom<&str> for WalletTxid {
    type Error = WalletCoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for WalletTxid {
    type Error = WalletCoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl fmt::Display for WalletTxid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Strongly-typed transaction virtual size in vbytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct VSize(pub u64);

impl VSize {
    /// Create a virtual-size wrapper.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Access the inner vsize value.
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns true when the size is zero.
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<VSize> for u64 {
    fn from(value: VSize) -> Self {
        value.0
    }
}

impl From<u64> for VSize {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for VSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} vB", self.0)
    }
}

/// Strongly-typed block height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BlockHeight(pub u32);

impl BlockHeight {
    /// Create a block-height wrapper.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Access the inner block height.
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<BlockHeight> for u32 {
    fn from(value: BlockHeight) -> Self {
        value.0
    }
}

impl From<u32> for BlockHeight {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl fmt::Display for BlockHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Strongly-typed percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Percent(pub u8);

impl Percent {
    /// Create a validated percentage value in the inclusive range 0..=100.
    pub fn new(value: u8) -> Result<Self, WalletCoreError> {
        if value > 100 {
            return Err(WalletCoreError::InvalidPercent(value.to_string()));
        }
        Ok(Self(value))
    }

    /// Access the inner percentage value.
    pub fn as_u8(self) -> u8 {
        self.0
    }
}

impl From<Percent> for u8 {
    fn from(value: Percent) -> Self {
        value.0
    }
}

impl From<u8> for Percent {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

/// Strongly-typed base64-encoded PSBT payload.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PsbtBase64(pub String);

impl PsbtBase64 {
    /// Create a PSBT-base64 wrapper.
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Borrow the inner PSBT base64 string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the wrapper and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
    pub fn to_psbt(&self) -> crate::WalletCoreResult<Psbt> {
        self.try_into()
    }
}

impl From<String> for PsbtBase64 {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for PsbtBase64 {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<PsbtBase64> for String {
    fn from(value: PsbtBase64) -> Self {
        value.0
    }
}

impl AsRef<str> for PsbtBase64 {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PsbtBase64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl TryFrom<&PsbtBase64> for Psbt {
    type Error = WalletCoreError;

    fn try_from(value: &PsbtBase64) -> Result<Self, Self::Error> {
        Psbt::from_str(value.as_str()).map_err(|e| WalletCoreError::InvalidPsbt(e.to_string()))
    }
}
/// Strongly-typed raw transaction hex payload.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TxHex(pub String);

impl TxHex {
    /// Create a tx-hex wrapper.
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Borrow the inner transaction hex string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the wrapper and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for TxHex {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TxHex {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<TxHex> for String {
    fn from(value: TxHex) -> Self {
        value.0
    }
}

impl AsRef<str> for TxHex {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TxHex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
