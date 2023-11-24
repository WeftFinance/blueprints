use crate::helpers::{init::TestHelper, methods::*};
use radix_engine_interface::{blueprints::consensus_manager::TimePrecision, prelude::*};

// ! ISSUE WITH TEST RUNNER: CANNOT MOVE TIME FORWARD
#[test]
fn test_liquidation() {
    let mut helper = TestHelper::new();

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
    // Borrow 400$  Of USD
    market_borrow(
        &mut helper,
        borrower_key,
        borrower_account,
        cdp_id,
        usd,
        dec!(419),
    )
    .expect_commit_success();

    // Change USD (in XRD) PRICE DROP FROM 25 to 10
    admin_update_price(&mut helper, 1u64, usd, dec!(27)).expect_commit_success();

    market_update_pool_state(&mut helper, usd).expect_commit_success();

    // SET UP LIQUIDATOR
    let (liquidator_user_key, _, liquidator_user_account) =
        helper.test_runner.new_allocated_account();

    let requested_collaterals: Vec<ResourceAddress> = vec![XRD];

    let xrd_balance = helper
        .test_runner
        .get_component_balance(liquidator_user_account, XRD);

    // SWAP Collateral XRD TO LOAN USD
    swap(
        &mut helper,
        liquidator_user_account,
        liquidator_user_key,
        xrd_balance,
        XRD,
        usd,
    )
    .expect_commit_success();

    let usd_balance_after_swap = helper
        .test_runner
        .get_component_balance(liquidator_user_account, usd);

    let mut payments: Vec<(ResourceAddress, Decimal)> = Vec::new();

    payments.push((usd, usd_balance_after_swap));

    let receipt = market_fast_liquidation(
        &mut helper,
        liquidator_user_key,
        liquidator_user_account,
        cdp_id,
        payments,
        requested_collaterals,
    );

    println!("{:?}", receipt);

    receipt.expect_commit_success();
}
