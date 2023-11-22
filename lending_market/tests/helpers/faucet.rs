use radix_engine::vm::NoExtension;
use radix_engine_interface::prelude::*;
use scrypto::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;
use std::path::Path;

use crate::helpers::init::build_and_dumb_to_fs;

use super::price_feed::PriceFeedTestHelper;

pub struct FaucetTestHelper {
    pub faucet_component_address: ComponentAddress,
    pub faucet_admin_badge: ResourceAddress,
    pub usdc_resource_address: ResourceAddress,
}
impl FaucetTestHelper {
    pub fn new(
        test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
        owner_account_address: ComponentAddress,
        owner_public_key: Secp256k1PublicKey,
        // owner_badge_resource_address: ResourceAddress,
        price_feed: &PriceFeedTestHelper,
    ) -> FaucetTestHelper {
        let faucet_package_address =
            test_runner.compile_and_publish(Path::new("../basic_resource_faucet"));

        //

        let manifest_builder_0 = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                faucet_package_address,
                "Faucet",
                "instantiate",
                manifest_args!(
                    // owner_badge_resource_address,
                    price_feed.price_feed_component_address
                ),
            )
            .deposit_batch(owner_account_address);

        let receipt_0 = test_runner.execute_manifest(
            build_and_dumb_to_fs(manifest_builder_0, "faucet_instantiate".into()),
            vec![NonFungibleGlobalId::from_public_key(&owner_public_key)],
        );
        let result_0 = receipt_0.expect_commit(true);

        let component_addresses_created = result_0.new_component_addresses();
        let faucet_component_address = component_addresses_created[0];

        let resource_addresses_created = result_0.new_resource_addresses();
        let faucet_admin_badge = resource_addresses_created[0];

        //

        let manifest_builder_1 = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                owner_account_address,
                NonFungibleGlobalId::new(faucet_admin_badge, NonFungibleLocalId::integer(1)),
            )
            .call_method(
                faucet_component_address,
                "create_resource",
                manifest_args!(
                    "USDC",
                    "USDC",
                    "https://res.cloudinary.com/daisvxhyu/image/upload/v1679440531/825_lkjddk.png",
                    dec!(1_000_000)
                ),
            )
            .deposit_batch(owner_account_address);

        let receipt_1 = test_runner.execute_manifest(
            build_and_dumb_to_fs(manifest_builder_1, "create_resource_USDC".into()),
            vec![NonFungibleGlobalId::from_public_key(&owner_public_key)],
        );
        let result_1 = receipt_1.expect_commit(true);

        let resource_addresses_created = result_1.new_resource_addresses();
        let usdc_resource_address = resource_addresses_created[0];

        assert_eq!(
            test_runner.get_component_balance(owner_account_address, usdc_resource_address),
            dec!(1_000_000)
        );

        //

        let manifest_builder_2 = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                owner_account_address,
                NonFungibleGlobalId::new(
                    price_feed.price_feed_admin_badge,
                    NonFungibleLocalId::integer(1),
                ),
            )
            .call_method(
                price_feed.price_feed_component_address,
                "admin_update_price",
                manifest_args!(XRD, dec!(1)),
            )
            .call_method(
                price_feed.price_feed_component_address,
                "admin_update_price",
                manifest_args!(usdc_resource_address, dec!(25)),
            );

        let _result_2 = test_runner
            .execute_manifest(
                build_and_dumb_to_fs(manifest_builder_2, "price_feed_update_price".into()),
                vec![NonFungibleGlobalId::from_public_key(&owner_public_key)],
            )
            .expect_commit(true);

        Self {
            faucet_component_address,
            faucet_admin_badge,
            usdc_resource_address,
        }
    }
}
