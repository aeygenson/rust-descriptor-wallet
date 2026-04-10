

use wallet_core::types::FeeRateSatPerVb;

#[test]
fn fee_rate_domain_logic_is_consistent() {
    let low = FeeRateSatPerVb::from(1);
    let high = FeeRateSatPerVb::from(10);

    assert!(high.as_u64() > low.as_u64());
}

#[test]
fn zero_fee_rate_is_invalid() {
    let zero = FeeRateSatPerVb::from(0);

    assert_eq!(zero.as_u64(), 0);
    assert!(zero.is_zero());
    assert!(zero.ensure_non_zero().is_err());
}

#[test]
fn non_zero_fee_rate_is_valid() {
    let fee = FeeRateSatPerVb::from(5);

    assert!(!fee.is_zero());
    assert!(fee.ensure_non_zero().is_ok());
}

#[test]
fn higher_fee_rate_compares_above_lower_fee_rate() {
    let original = FeeRateSatPerVb::from(2);
    let requested = FeeRateSatPerVb::from(5);

    assert!(requested.as_u64() > original.as_u64());
}

#[test]
fn equal_fee_rates_do_not_form_a_strict_bump() {
    let original = FeeRateSatPerVb::from(5);
    let requested = FeeRateSatPerVb::from(5);

    assert!(!(requested.as_u64() > original.as_u64()));
}

#[test]
fn lower_fee_rate_does_not_form_a_strict_bump() {
    let original = FeeRateSatPerVb::from(5);
    let requested = FeeRateSatPerVb::from(2);

    assert!(!(requested.as_u64() > original.as_u64()));
}