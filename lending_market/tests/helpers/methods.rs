use super::init::{build_and_dumb_to_fs, TestHelper};
use radix_engine_interface::prelude::*;
use scrypto::*;
use scrypto_test::prelude::*;

pub fn get_resource(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    xrd_amount: Decimal,
) -> TransactionReceipt {
    let manifest_builder_0 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(user_account_address, XRD, xrd_amount)
        .take_all_from_worktop(XRD, "xrd_buket")
        .with_name_lookup(|builder, lookup| {
            let xrd_buket = lookup.bucket("xrd_buket");
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

pub fn market_contribute(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    res_address: ResourceAddress,
    amount: Decimal,
) -> TransactionReceipt {
    let manifest_builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(user_account_address, res_address, amount)
        .take_all_from_worktop(res_address, "res_buket")
        .with_name_lookup(|builder, lookup| {
            let bucket = lookup.bucket("res_buket");

            builder.call_method(
                helper.market.market_component_address,
                "contribute",
                manifest_args!(bucket),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "contribute".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_redeem(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    res_address: ResourceAddress,
    amount: Decimal,
) -> TransactionReceipt {
    let manifest_builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(user_account_address, res_address, amount)
        .take_all_from_worktop(res_address, "res_buket")
        .with_name_lookup(|builder, lookup| {
            let bucket = lookup.bucket("res_buket");

            builder.call_method(
                helper.market.market_component_address,
                "redeem",
                manifest_args!(bucket),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "redeem".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_create_cdp(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    deposits: Vec<(ResourceAddress, Decimal)>,
) -> TransactionReceipt {
    let mut bukets = Vec::<ManifestBucket>::new();

    let manifest_builder_0 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .with_name_lookup(|builder, _lookup| {
            let (_, newbuilder) =
                deposits
                    .iter()
                    .fold((0, builder), |(i, builder), (res_address, amount)| {
                        (
                            i,
                            builder
                                .withdraw_from_account(user_account_address, *res_address, *amount)
                                .take_all_from_worktop(*res_address, format!("res_buket_{}", i))
                                .with_name_lookup(|builder, lookup| {
                                    bukets.push(lookup.bucket(format!("res_buket_{}", i)));

                                    builder
                                }),
                        )
                    });

            newbuilder
        })
        .call_method(
            helper.market.market_component_address,
            "create_cdp",
            manifest_args!(None::<Decimal>, None::<Decimal>, None::<Decimal>, bukets),
        )
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder_0, "create_cdp".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_deposit(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    cdp_id: u64,
    res_address: ResourceAddress,
    amount: Decimal,
) -> TransactionReceipt {
    let manifest_builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(
            user_account_address,
            NonFungibleGlobalId::new(
                helper.market.cdp_resource_address,
                NonFungibleLocalId::Integer(cdp_id.into()),
            ),
        )
        .pop_from_auth_zone("cdp_proof")
        .withdraw_from_account(user_account_address, res_address, amount)
        .take_all_from_worktop(res_address, "res_buket")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("cdp_proof");
            let bucket = lookup.bucket("res_buket");

            builder.call_method(
                helper.market.market_component_address,
                "deposit",
                manifest_args!(proof, vec![bucket]),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "deposit".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_withdraw(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    cdp_id: u64,
    res_address: ResourceAddress,
    amount: Decimal,
    keep_pool_units: bool,
) -> TransactionReceipt {
    let manifest_builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(
            user_account_address,
            NonFungibleGlobalId::new(
                helper.market.cdp_resource_address,
                NonFungibleLocalId::Integer(cdp_id.into()),
            ),
        )
        .pop_from_auth_zone("cdp_proof")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("cdp_proof");

            builder.call_method(
                helper.market.market_component_address,
                "withdraw",
                manifest_args!(proof, vec![(res_address, amount, keep_pool_units)]),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "withdraw".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_borrow(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    cdp_id: u64,
    res_address: ResourceAddress,
    amount: Decimal,
) -> TransactionReceipt {
    let manifest_builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(
            user_account_address,
            NonFungibleGlobalId::new(
                helper.market.cdp_resource_address,
                NonFungibleLocalId::Integer(cdp_id.into()),
            ),
        )
        .pop_from_auth_zone("cdp_proof")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("cdp_proof");

            builder.call_method(
                helper.market.market_component_address,
                "borrow",
                manifest_args!(proof, vec![(res_address, amount)]),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "borrow".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_repay(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    cdp_id: u64,
    res_address: ResourceAddress,
    amount: Decimal,
) -> TransactionReceipt {
    let manifest_builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(
            user_account_address,
            NonFungibleGlobalId::new(
                helper.market.cdp_resource_address,
                NonFungibleLocalId::Integer(cdp_id.into()),
            ),
        )
        .pop_from_auth_zone("cdp_proof")
        .withdraw_from_account(user_account_address, res_address, amount)
        .take_all_from_worktop(res_address, "res_buket")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("cdp_proof");
            let bucket = lookup.bucket("res_buket");

            builder.call_method(
                helper.market.market_component_address,
                "repay",
                manifest_args!(proof, None::<NonFungibleLocalId>, vec![bucket]),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "repay".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

// fn generic_txm(manifest_builder: ManifestBuilder) -> ManifestBuilder {
//     manifest_builder
//         .lock_fee_from_faucet()
//         .create_proof_from_account_of_non_fungible(
//             user_account_address,
//             NonFungibleGlobalId::new(
//                 helper.market.cdp_resource_address,
//                 NonFungibleLocalId::Integer(cdp_id.into()),
//             ),
//         )
//         .pop_from_auth_zone("cdp_proof")
//         .withdraw_from_account(user_account_address, res_address, amount)
//         .take_all_from_worktop(res_address, "res_buket");

//     manifest_builder
// }
// fn generic_cdp_txm() {}
