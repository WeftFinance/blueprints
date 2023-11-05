use lending_market::modules::pool_config::*;
use scrypto::*;
use scrypto_test::prelude::*;

#[test]
fn test_check_valid_config() {
    let config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: 3600,
    };

    assert!(config.check().is_ok());
}

#[test]
fn test_check_invalid_lending_fee_rate() {
    let config = PoolConfig {
        lending_fee_rate: dec!(-0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: 3600,
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_flashloan_fee_rate() {
    let config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(-0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: 3600,
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_deposit_limit() {
    let config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(-100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: 3600,
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_borrow_limit() {
    let config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(-1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: 3600,
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_utilization_limit() {
    let config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(-0.8)),
        price_update_period: 3600,
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_liquidation_bonus_rate() {
    let config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(-0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: 3600,
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_loan_close_factor() {
    let config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(-0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: 3600,
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_price_update_period() {
    let config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: -3600,
    };

    assert!(config.check().is_err());
}

#[test]
fn test_update_config_valid_input() {
    let mut config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: 3600,
    };

    let input = UpdatePoolConfigInput::LendingFeeRate(dec!(0.02));
    assert!(config.update_config(input).is_ok());
    assert_eq!(config.lending_fee_rate, dec!(0.02));
}

#[test]
fn test_update_config_invalid_input() {
    let mut config = PoolConfig {
        lending_fee_rate: dec!(0.01),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: Some(dec!(100)),
        borrow_limit: Some(dec!(1000)),
        utilization_limit: Some(dec!(0.8)),
        price_update_period: 3600,
    };

    let input = UpdatePoolConfigInput::LendingFeeRate(dec!(-0.01));
    assert!(config.update_config(input).is_err());
    assert_eq!(config.lending_fee_rate, dec!(0.01));
}
