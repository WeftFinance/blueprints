use lending_market::modules::operation_status::{OperatingStatus, OperatingStatusInput};
use scrypto::*;

#[test]
fn new_operating_status() {
    let operating_status = OperatingStatus::default();
    assert_eq!(operating_status.is_contribute_enabled, true);
    assert_eq!(operating_status.is_redeem_enabled, true);
    assert_eq!(operating_status.is_deposit_enabled, true);
    assert_eq!(operating_status.is_withdraw_enabled, true);
    assert_eq!(operating_status.is_borrow_enabled, true);
    assert_eq!(operating_status.is_repay_enabled, true);
    assert_eq!(operating_status.is_refinance_enabled, true);
    assert_eq!(operating_status.is_liquidate_enabled, true);
    assert_eq!(operating_status.is_flashloan_enabled, true);
}

#[test]
fn update_operating_status() {
    let mut operating_status = OperatingStatus::default();

    operating_status.update(OperatingStatusInput::Contribute(true));
    assert_eq!(operating_status.is_contribute_enabled, true);

    operating_status.update(OperatingStatusInput::Redeem(true));
    assert_eq!(operating_status.is_redeem_enabled, true);

    operating_status.update(OperatingStatusInput::Deposit(true));
    assert_eq!(operating_status.is_deposit_enabled, true);

    operating_status.update(OperatingStatusInput::Withdraw(true));
    assert_eq!(operating_status.is_withdraw_enabled, true);

    operating_status.update(OperatingStatusInput::Borrow(true));
    assert_eq!(operating_status.is_borrow_enabled, true);

    operating_status.update(OperatingStatusInput::Repay(true));
    assert_eq!(operating_status.is_repay_enabled, true);

    operating_status.update(OperatingStatusInput::Refinance(true));
    assert_eq!(operating_status.is_refinance_enabled, true);

    operating_status.update(OperatingStatusInput::Liquidation(true));
    assert_eq!(operating_status.is_liquidate_enabled, true);

    operating_status.update(OperatingStatusInput::Flashloan(true));
    assert_eq!(operating_status.is_flashloan_enabled, true);

    operating_status.update(OperatingStatusInput::Contribute(false));
    assert_eq!(operating_status.is_contribute_enabled, false);

    operating_status.update(OperatingStatusInput::Redeem(false));
    assert_eq!(operating_status.is_redeem_enabled, false);

    operating_status.update(OperatingStatusInput::Deposit(false));
    assert_eq!(operating_status.is_deposit_enabled, false);

    operating_status.update(OperatingStatusInput::Withdraw(false));
    assert_eq!(operating_status.is_withdraw_enabled, false);

    operating_status.update(OperatingStatusInput::Borrow(false));
    assert_eq!(operating_status.is_borrow_enabled, false);

    operating_status.update(OperatingStatusInput::Repay(false));
    assert_eq!(operating_status.is_repay_enabled, false);

    operating_status.update(OperatingStatusInput::Refinance(false));
    assert_eq!(operating_status.is_refinance_enabled, false);

    operating_status.update(OperatingStatusInput::Liquidation(false));
    assert_eq!(operating_status.is_liquidate_enabled, false);

    operating_status.update(OperatingStatusInput::Flashloan(false));
    assert_eq!(operating_status.is_flashloan_enabled, false);
}

#[test]
fn check_contribute_operating_status() {
    let mut operating_status = OperatingStatus::default();

    operating_status.update(OperatingStatusInput::Contribute(true));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Contribute(true)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Contribute(false)),
        false
    );

    operating_status.update(OperatingStatusInput::Redeem(false));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Redeem(false)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Redeem(true)),
        false
    );
}

#[test]
fn check_redeem_operating_status() {
    let mut operating_status = OperatingStatus::default();

    operating_status.update(OperatingStatusInput::Redeem(true));

    assert_eq!(
        operating_status.check(OperatingStatusInput::Redeem(true)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Redeem(false)),
        false
    );

    operating_status.update(OperatingStatusInput::Contribute(false));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Contribute(false)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Contribute(true)),
        false
    );
}

#[test]
fn check_deposit_operating_status() {
    let mut operating_status = OperatingStatus::default();

    operating_status.update(OperatingStatusInput::Deposit(true));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Deposit(true)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Deposit(false)),
        false
    );

    operating_status.update(OperatingStatusInput::Withdraw(false));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Withdraw(false)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Withdraw(true)),
        false
    );
}

#[test]
fn check_withdraw_operating_status() {
    let mut operating_status = OperatingStatus::default();

    operating_status.update(OperatingStatusInput::Withdraw(true));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Withdraw(true)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Withdraw(false)),
        false
    );

    operating_status.update(OperatingStatusInput::Deposit(false));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Deposit(false)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Deposit(true)),
        false
    );
}

#[test]
fn check_borrow_operating_status() {
    let mut operating_status = OperatingStatus::default();

    operating_status.update(OperatingStatusInput::Borrow(true));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Borrow(true)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Borrow(false)),
        false
    );

    operating_status.update(OperatingStatusInput::Repay(false));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Repay(false)),
        true
    );

    assert_eq!(
        operating_status.check(OperatingStatusInput::Repay(true)),
        false
    );
}

#[test]
fn check_repay_operating_status() {
    let mut operating_status = OperatingStatus::default();

    operating_status.update(OperatingStatusInput::Repay(true));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Repay(true)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Repay(false)),
        false
    );

    operating_status.update(OperatingStatusInput::Borrow(false));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Borrow(false)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Borrow(true)),
        false
    );
}

#[test]
fn check_refinance_operating_status() {
    let mut operating_status = OperatingStatus::default();

    operating_status.update(OperatingStatusInput::Refinance(true));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Refinance(true)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Refinance(false)),
        false
    );

    operating_status.update(OperatingStatusInput::Liquidation(false));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Liquidation(false)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Liquidation(true)),
        false
    );
}

#[test]
fn check_liquidate_operating_status() {
    let mut operating_status = OperatingStatus::default();

    operating_status.update(OperatingStatusInput::Liquidation(true));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Liquidation(true)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Liquidation(false)),
        false
    );

    operating_status.update(OperatingStatusInput::Refinance(false));
    assert_eq!(
        operating_status.check(OperatingStatusInput::Refinance(false)),
        true
    );
    assert_eq!(
        operating_status.check(OperatingStatusInput::Refinance(true)),
        false
    );
}
