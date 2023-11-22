use crate::helpers::{init::TestHelper, methods::*};
use radix_engine_interface::prelude::*;

#[test]
pub fn test_flash_loan() {
    let mut helper = TestHelper::new();
    // SET UP A LP PROVIDER
    let (lp_user_key, _, lp_user_account) = helper.test_runner.new_allocated_account();
    helper.test_runner.load_account_from_faucet(lp_user_account);
    helper.test_runner.load_account_from_faucet(lp_user_account);

    // Provide 25000 XRD
    market_contribute(&mut helper, lp_user_key, lp_user_account, XRD, dec!(25_000))
        .expect_commit_success();

    let (user_public_key, _, user_account_address) = helper.test_runner.new_allocated_account();
    helper
        .test_runner
        .load_account_from_faucet(user_account_address);
    let mut loan_amounts: IndexMap<ResourceAddress, Decimal> = IndexMap::new();
    let loan_amount = Decimal::from(1000);
    loan_amounts.insert(XRD, loan_amount);

    market_take_batch_flashloan(
        &mut helper,
        user_public_key,
        user_account_address,
        loan_amounts,
    );

    get_resource_flash_loan(
        &mut helper,
        user_public_key,
        user_account_address,
        loan_amount,
    );

    let mut payments: Vec<(ResourceAddress, Decimal)> = Vec::new();
    payments.push((XRD, loan_amount + Decimal::from(100)));
    market_repay_batch_flashloan(&mut helper, user_public_key, user_account_address, payments)
        .expect_commit_success();
}
