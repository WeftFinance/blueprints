use crate::helpers::{
    faucet::FaucetTestHelper, init::TestHelper, market::MarketTestHelper, methods::*,
    price_feed::PriceFeedTestHelper,
};
use radix_engine_interface::{blueprints::consensus_manager::TimePrecision, prelude::*};
use scrypto_unit::*;
use std::path::Path;

// ! ISSUE WITH TEST RUNNER: CANNOT MOVE TIME FORWARD
#[test]
fn test_liquidation() {
    let mut helper = TestHelper::new();

    let epoch = helper.test_runner.get_current_epoch();

    print!(
        "Begin {:?}",
        helper.test_runner.get_current_time(TimePrecision::Minute)
    );

    // SET UP A LP PROVIDER
    let (lp_user_key, _, lp_user_account) = helper.test_runner.new_allocated_account();
    helper.test_runner.load_account_from_faucet(lp_user_account);
    helper.test_runner.load_account_from_faucet(lp_user_account);
    get_resource(&mut helper, lp_user_key, lp_user_account, dec!(25_000)) //
        .expect_commit_success();

    let usd = helper.faucet.usdc_resource_address;

    assert_eq!(
        helper
            .test_runner
            .get_component_balance(lp_user_account, usd),
        dec!(1_000)
    );

    market_contribute(&mut helper, lp_user_key, lp_user_account, usd, dec!(800))
        .expect_commit_success();

    assert_eq!(
        helper
            .test_runner
            .get_component_balance(lp_user_account, usd),
        dec!(200)
    );

    // SET UP A BORROWER
    let (borrower_key, _, borrower_account) = helper.test_runner.new_allocated_account();
    helper
        .test_runner
        .load_account_from_faucet(borrower_account);

    //Create CDP WITH 15000 XRD AS Collateral <=> 600$
    market_create_cdp(
        &mut helper,
        borrower_key,
        borrower_account,
        vec![(XRD, dec!(15_000))],
    ) //
    .expect_commit_success();

    let usd = helper.faucet.usdc_resource_address;

    let cdp_id: u64 = 1;
    // // Borrow 400$  Of USD
    market_borrow(
        &mut helper,
        borrower_key,
        borrower_account,
        cdp_id,
        usd,
        dec!(400),
    )
    .expect_commit_success();

    helper.test_runner.set_current_epoch(epoch.next().unwrap());

    print!(
        "After Borrow {:?}",
        helper.test_runner.get_current_time(TimePrecision::Minute)
    );

    // Change XRD PRICE DROP FROM 0.04 to 0.02
    admin_update_price(&mut helper, 1u64, XRD, dec!(0.02)).expect_commit_success();

    helper.test_runner.set_current_epoch(epoch.next().unwrap());

    get_price(&mut helper, XRD).expect_commit_success();

    // SET UP LIQUIDATOR
    let (liquidator_user_key, _, liquidator_user_account) =
        helper.test_runner.new_allocated_account();

    let mut requested_collaterals: Vec<ResourceAddress> = Vec::new();
    requested_collaterals.push(XRD);

    // // START LIQUIDATION
    market_start_liquidation(
        &mut helper,
        liquidator_user_key,
        liquidator_user_account,
        cdp_id,
        requested_collaterals,
        None::<Decimal>,
    )
    .expect_commit_failure();

    market_update_pool_state(&mut helper, XRD);

    // let xrd_balance = helper
    //     .test_runner
    //     .get_component_balance(liquidator_user_account, XRD)
    //     - Decimal::from(10_000);

    // let usd_balance_before_swap = helper
    //     .test_runner
    //     .get_component_balance(liquidator_user_account, usd);

    // // SWAP Collateral XRD TO LOAN USD
    // swap(
    //     &mut helper,
    //     liquidator_user_account,
    //     liquidator_user_key,
    //     xrd_balance,
    //     XRD,
    //     usd,
    // ).expect_commit_success();

    // let usd_balance_after_swap = helper
    //     .test_runner
    //     .get_component_balance(liquidator_user_account, usd);

    // let mut payments: Vec<(ResourceAddress, Decimal)> = Vec::new();

    // payments.push((usd, usd_balance_after_swap - usd_balance_before_swap));

    // market_end_liquidation(
    //     &mut helper,
    //     liquidator_user_key,
    //     liquidator_user_account,
    //     payments,
    // ).expect_commit_success();
}
