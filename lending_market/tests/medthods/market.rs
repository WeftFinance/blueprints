use crate::helpers::init::*;
use radix_engine_interface::prelude::*;
use scrypto::*;
use scrypto_test::prelude::*;

pub fn market_update_pool_state(
    helper: &mut TestHelper,
    res_address: ResourceAddress,
) -> TransactionReceiptV1 {
    let manifest = ManifestBuilder::new().lock_fee_from_faucet().call_method(
        helper.market.market_component_address,
        "update_pool_state",
        manifest_args!(res_address, true, true),
    );

    let receipt = helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest, "update_pool_state".into()),
        vec![NonFungibleGlobalId::from_public_key(
            &helper.owner_public_key,
        )],
    );

    receipt.expect_commit_success();

    println!("{:?}\n", receipt);

    receipt
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
        .take_all_from_worktop(res_address, "res_bucket")
        .with_name_lookup(|builder, lookup| {
            let bucket = lookup.bucket("res_bucket");

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
        .take_all_from_worktop(res_address, "res_bucket")
        .with_name_lookup(|builder, lookup| {
            let bucket = lookup.bucket("res_bucket");

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
    let mut buckets = Vec::<ManifestBucket>::new();

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
                                .take_all_from_worktop(*res_address, format!("res_bucket_{}", i))
                                .with_name_lookup(|builder, lookup| {
                                    buckets.push(lookup.bucket(format!("res_bucket_{}", i)));

                                    builder
                                }),
                        )
                    });

            newbuilder
        })
        .call_method(
            helper.market.market_component_address,
            "create_cdp",
            manifest_args!(None::<Decimal>, None::<Decimal>, None::<Decimal>, buckets),
        )
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder_0, "create_cdp".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_add_collateral(
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
        .take_all_from_worktop(res_address, "res_bucket")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("cdp_proof");
            let bucket = lookup.bucket("res_bucket");

            builder.call_method(
                helper.market.market_component_address,
                "add_collateral",
                manifest_args!(proof, vec![bucket]),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "add_collateral".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_remove_collateral(
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
                "remove_collateral",
                manifest_args!(proof, vec![(res_address, amount, keep_pool_units)]),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "remove_collateral".into()),
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
        .take_all_from_worktop(res_address, "res_bucket")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("cdp_proof");
            let bucket = lookup.bucket("res_bucket");

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

pub fn market_start_liquidation(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    cdp_id: u64,
    requested_collaterals: Vec<ResourceAddress>,
    total_payment_value: Option<Decimal>,
) -> TransactionReceipt {
    let manifest_builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            helper.market.market_component_address,
            "start_liquidation",
            manifest_args!(
                NonFungibleLocalId::integer(cdp_id),
                requested_collaterals,
                total_payment_value
            ),
        )
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "start_liquidation".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_end_liquidation(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    payments: Vec<(ResourceAddress, Decimal)>,
) -> TransactionReceipt {
    let mut payment_buckets = Vec::<ManifestBucket>::new();
    let manifest_builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .take_all_from_worktop(
            helper.market.liquidation_term_resource_address,
            "liquidation_term",
        )
        .with_name_lookup(|builder, _lookup| {
            let liquidation_term_bucket = _lookup.bucket("liquidation_term_bucket");
            let (_, newbuilder) =
                payments
                    .iter()
                    .fold((0, builder), |(i, builder), (res_address, amount)| {
                        (
                            i,
                            builder
                                .withdraw_from_account(user_account_address, *res_address, *amount)
                                .take_all_from_worktop(
                                    *res_address,
                                    format!("payment_bucket_{}", i),
                                )
                                .with_name_lookup(|builder, lookup| {
                                    payment_buckets
                                        .push(lookup.bucket(format!("payment_bucket_{}", i)));
                                    builder
                                }),
                        )
                    });

            newbuilder.call_method(
                helper.market.market_component_address,
                "end_liquidation",
                manifest_args!(payment_buckets, liquidation_term_bucket),
            )
        });

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "end_liquidation".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_fast_liquidation(
    helper: &mut TestHelper,
    user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    cdp_id: u64,
    payments: Vec<(ResourceAddress, Decimal)>,
    requested_collaterals: Vec<ResourceAddress>,
) -> TransactionReceipt {
    let mut payment_buckets = Vec::<ManifestBucket>::new();
    let manifest_builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .with_name_lookup(|builder, _lookup| {
            let (_, newbuilder) =
                payments
                    .iter()
                    .fold((0, builder), |(i, builder), (res_address, amount)| {
                        (
                            i,
                            builder
                                .withdraw_from_account(user_account_address, *res_address, *amount)
                                .take_all_from_worktop(
                                    *res_address,
                                    format!("payment_bucket_{}", i),
                                )
                                .with_name_lookup(|builder, lookup| {
                                    payment_buckets
                                        .push(lookup.bucket(format!("payment_bucket_{}", i)));
                                    builder
                                }),
                        )
                    });

            newbuilder.call_method(
                helper.market.market_component_address,
                "fast_liquidation",
                manifest_args!(
                    NonFungibleLocalId::integer(cdp_id),
                    payment_buckets,
                    requested_collaterals
                ),
            )
        })
        .deposit_batch(user_account_address);

    helper.test_runner.execute_manifest(
        build_and_dumb_to_fs(manifest_builder, "start_liquidation".into()),
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    )
}

pub fn market_take_batch_flashloan(
    helper: &mut TestHelper,
    _user_public_key: Secp256k1PublicKey,
    _user_account_address: ComponentAddress,
    loan_amounts: IndexMap<ResourceAddress, Decimal>,
    manifest_builder: ManifestBuilder,
) {
    manifest_builder.lock_fee_from_faucet().call_method(
        helper.market.market_component_address,
        "take_batch_flashloan",
        manifest_args!(loan_amounts),
    );
}

pub fn market_repay_batch_flashloan(
    helper: &mut TestHelper,
    _user_public_key: Secp256k1PublicKey,
    user_account_address: ComponentAddress,
    payments: Vec<(ResourceAddress, Decimal)>,
    manifest_builder: ManifestBuilder,
) {
    let mut payment_buckets = Vec::<ManifestBucket>::new();
    manifest_builder
        .lock_fee_from_faucet()
        .take_from_worktop(
            helper.market.batch_flashloan_resource_address,
            Decimal::from(1),
            "flash_loan_term_bucket",
        )
        .with_name_lookup(|builder, _lookup| {
            let flash_loan_term_bucket = _lookup.bucket("flash_loan_term_bucket");
            let (_, newbuilder) =
                payments
                    .iter()
                    .fold((0, builder), |(i, builder), (res_address, amount)| {
                        (
                            i,
                            builder
                                .withdraw_from_account(user_account_address, *res_address, *amount)
                                .take_all_from_worktop(
                                    *res_address,
                                    format!("payment_bucket_{}", i),
                                )
                                .with_name_lookup(|builder, lookup| {
                                    payment_buckets
                                        .push(lookup.bucket(format!("payment_bucket_{}", i)));
                                    builder
                                }),
                        )
                    });

            newbuilder
                .call_method(
                    helper.market.market_component_address,
                    "repay_batch_flashloan",
                    manifest_args!(payment_buckets, flash_loan_term_bucket),
                )
                .deposit_batch(user_account_address)
        });
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
