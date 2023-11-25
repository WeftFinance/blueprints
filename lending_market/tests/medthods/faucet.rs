use crate::helpers::init::*;
use radix_engine_interface::prelude::*;
use scrypto::*;
use scrypto_test::prelude::*;

pub fn faucet_get_resource(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    xrd_amount: Decimal,
) -> TransactionReceipt {
    let manifest_builder_0 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(user_account_address, XRD, xrd_amount)
        .take_all_from_worktop(XRD, "xrd_bucket")
        .with_name_lookup(|builder, lookup| {
            let xrd_buket = lookup.bucket("xrd_bucket");
            builder.call_method(
                helper.faucet.faucet_component_address,
                "get_resource",
                manifest_args!(helper.faucet.usdc_resource_address, xrd_buket),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder_0, "get_resource".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn faucet_swap(
    helper: &mut TestHelper,
    account_address: ComponentAddress,
    account_public_key: Secp256k1PublicKey,
    from_amount: Decimal,
    from_resource_address: ResourceAddress,
    to_resource_address: ResourceAddress,
) -> TransactionReceiptV1 {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account_address, from_resource_address, from_amount)
        .take_all_from_worktop(from_resource_address, "from_tokens")
        .with_name_lookup(|builder, lookup: ManifestNameLookup| {
            let bucket = lookup.bucket("from_tokens");
            builder
                .call_method(
                    helper.faucet.faucet_component_address,
                    "swap",
                    manifest_args!(bucket, to_resource_address),
                )
                .deposit_batch(account_address)
        })
        .build();

    let receipt = helper.test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account_public_key)],
    );

    println!("{:?}\n", receipt);

    receipt
}
