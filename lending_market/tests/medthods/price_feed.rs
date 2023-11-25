use crate::helpers::init::*;
use radix_engine_interface::prelude::*;
use scrypto::*;
use scrypto_test::prelude::*;

pub fn price_feed_admin_update_price(
    helper: &mut TestHelper,
    admin_non_fungible_id: u64,
    resource_address: ResourceAddress,
    price: Decimal,
) -> TransactionReceiptV1 {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(
            helper.owner_account_address,
            NonFungibleGlobalId::new(
                helper.price_feed.price_feed_admin_badge,
                NonFungibleLocalId::integer(admin_non_fungible_id),
            ),
        )
        .call_method(
            helper.price_feed.price_feed_component_address,
            "admin_update_price",
            manifest_args!(resource_address, price),
        )
        .deposit_batch(helper.owner_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest, "admin_update_price".into()),
        vec![NonFungibleGlobalId::from_public_key(
            &helper.owner_public_key,
        )],
    )
}

pub fn price_feed_get_price(
    helper: &mut TestHelper,
    resource_address: ResourceAddress,
) -> TransactionReceiptV1 {
    let manifest = ManifestBuilder::new().lock_fee_from_faucet().call_method(
        helper.price_feed.price_feed_component_address,
        "get_price",
        manifest_args!(resource_address),
    );

    let receipt = helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest, "get_price".into()),
        vec![NonFungibleGlobalId::from_public_key(
            &helper.owner_public_key,
        )],
    );

    receipt.expect_commit_success();

    println!("{:?}\n", receipt);

    receipt
}
