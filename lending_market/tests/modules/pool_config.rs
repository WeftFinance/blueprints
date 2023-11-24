use lending_market::modules::pool_config::*;
use scrypto::*;
use scrypto_test::prelude::*;

fn get_default_pool_config() -> PoolConfig {
    PoolConfig {
        protocol_interest_fee_rate: dec!(0.01),
        protocol_flashloan_fee_rate: dec!(0.005),
        protocol_liquidation_fee_rate: dec!(0.05),
        flashloan_fee_rate: dec!(0.005),
        asset_type: 1,
        liquidation_bonus_rate: dec!(0.05),
        loan_close_factor: dec!(0.5),
        deposit_limit: None,
        borrow_limit: None,
        utilization_limit: None,
        price_update_period: 3600,
        interest_update_period: 3600,
        price_expiration_period: 3601,
    }
}

#[test]
fn test_check_valid_config() {
    let config = get_default_pool_config();
    assert!(config.check().is_ok());
}

#[test]
fn test_check_invalid_protocol_interest_fee_rate() {
    let config = PoolConfig {
        protocol_interest_fee_rate: dec!(-0.01),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());

    let config = PoolConfig {
        protocol_interest_fee_rate: dec!(1.01),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_protocol_flashloan_fee_rate() {
    let config = PoolConfig {
        protocol_flashloan_fee_rate: dec!(-0.01),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());

    let config = PoolConfig {
        protocol_flashloan_fee_rate: dec!(1.01),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_protocol_liquidation_fee_rate() {
    let config = PoolConfig {
        protocol_liquidation_fee_rate: dec!(-0.01),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());

    let config = PoolConfig {
        protocol_liquidation_fee_rate: dec!(1.01),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_flashloan_fee_rate() {
    let config = PoolConfig {
        flashloan_fee_rate: dec!(-0.005),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());

    let config = PoolConfig {
        flashloan_fee_rate: dec!(1.005),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_deposit_limit() {
    let config = PoolConfig {
        deposit_limit: Some(dec!(-100)),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_borrow_limit() {
    let config = PoolConfig {
        borrow_limit: Some(dec!(-1000)),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_utilization_limit() {
    let config = PoolConfig {
        utilization_limit: Some(dec!(-0.8)),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());

    let config = PoolConfig {
        utilization_limit: Some(dec!(1.2)),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_liquidation_bonus_rate() {
    let config = PoolConfig {
        liquidation_bonus_rate: dec!(-0.05),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());

    let config = PoolConfig {
        liquidation_bonus_rate: dec!(1.05),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_loan_close_factor() {
    let config = PoolConfig {
        loan_close_factor: dec!(-0.5),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());

    let config = PoolConfig {
        loan_close_factor: dec!(1.5),
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_price_update_period() {
    let config = PoolConfig {
        price_update_period: -3600,
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_interest_update_period() {
    let config = PoolConfig {
        interest_update_period: -3600,
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}

#[test]
fn test_check_invalid_price_expiration_period() {
    let config = PoolConfig {
        price_expiration_period: -3600,
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());

    let config = PoolConfig {
        price_update_period: 3600,
        price_expiration_period: 3600,
        ..get_default_pool_config()
    };

    assert!(config.check().is_err());
}
