use radix_engine::vm::NoExtension;
use radix_engine_interface::prelude::*;
use scrypto::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;
use std::path::Path;

pub struct PriceFeedTestHelper {
    pub price_feed_component_address: ComponentAddress,
    pub price_feed_admin_badge: ResourceAddress,
    pub price_feed_updater_badge: ResourceAddress,
}

impl PriceFeedTestHelper {
    pub fn new(
        test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
        owner_account_address: ComponentAddress,
        owner_public_key: Secp256k1PublicKey,
        // owner_badge_resource_address: ResourceAddress,
    ) -> PriceFeedTestHelper {
        let oracle_package_address =
            test_runner.compile_and_publish(Path::new("../internal_price_feed"));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                oracle_package_address,
                "PriceFeed",
                "instantiate",
                manifest_args!(),
            )
            .deposit_batch(owner_account_address)
            .build();

        let receipt = test_runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&owner_public_key)],
        );
        println!("{:?}\n", receipt);
        let result = receipt.expect_commit(true);

        let component_addresses_created = result.new_component_addresses();
        let price_feed_component_address = component_addresses_created[0];

        let resource_addresses_created = result.new_resource_addresses();
        let price_feed_admin_badge = resource_addresses_created[0];
        let price_feed_updater_badge = resource_addresses_created[1];

        Self {
            price_feed_component_address,
            price_feed_admin_badge,
            price_feed_updater_badge,
        }
    }
}
