use crate::helpers::{
    faucet::FaucetTestHelper, market::MarketTestHelper, price_feed::PriceFeedTestHelper,
};
use scrypto_unit::*;
use std::path::Path;

#[test]
fn test_instantiate_price_feed() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (owner_public_key, _, owner_account_address) = test_runner.new_allocated_account();
    let _helper =
        PriceFeedTestHelper::new(&mut test_runner, owner_account_address, owner_public_key);
}

#[test]
fn test_instantiate_faucet() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (owner_public_key, _, owner_account_address) = test_runner.new_allocated_account();
    let price_feed_helper =
        PriceFeedTestHelper::new(&mut test_runner, owner_account_address, owner_public_key);

    let _helper = FaucetTestHelper::new(
        &mut test_runner,
        owner_account_address,
        owner_public_key,
        &price_feed_helper,
    );
}

#[test]
fn test_instantiate_market() {
    let mut test_runner = TestRunnerBuilder::new().build();

    let (owner_public_key, _, owner_account_address) = test_runner.new_allocated_account();

    let price_feed_helper =
        PriceFeedTestHelper::new(&mut test_runner, owner_account_address, owner_public_key);

    let faucet_helper = FaucetTestHelper::new(
        &mut test_runner,
        owner_account_address,
        owner_public_key,
        &price_feed_helper,
    );

    let _helper = MarketTestHelper::new(
        &mut test_runner,
        owner_account_address,
        owner_public_key,
        &price_feed_helper,
        &faucet_helper,
    );
}

#[test]
fn test_create_pool_package_address() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let _pool_package_address =
        test_runner.compile_and_publish(Path::new("../single_resource_pool"));
    println!("{:?}\n", _pool_package_address);
}
