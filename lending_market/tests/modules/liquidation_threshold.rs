use lending_market::modules::liquidation_threshold::LiquidationThreshold;
use scrypto::*;
use scrypto_test::prelude::*;

fn create_sample_liquidation_threshold() -> LiquidationThreshold {
    let res_a_1 = SECP256K1_SIGNATURE_VIRTUAL_BADGE;
    let res_a_2 = ED25519_SIGNATURE_VIRTUAL_BADGE;
    let res_a_3 = PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE;

    let asset_type_1 = 0;
    let asset_type_2 = 1;
    let asset_type_3 = 2;

    LiquidationThreshold {
        identical_resource: Some(dec!(0.8)),
        identical_asset_type: Some(dec!(0.1)),
        resource: indexmap! {
            res_a_1 => dec!(0.2),
            res_a_2 => dec!(0.3),
            res_a_3 => dec!(0.4),
        },
        asset_type: indexmap! {
            asset_type_1 => dec!(0.5),
            asset_type_2 => dec!(0.6),
            asset_type_3 => dec!(0.7),
        },
        default_value: dec!(0),
    }
}

#[test]
fn test_check_valid_threshold() {
    let liquidation_threshold = create_sample_liquidation_threshold();
    assert!(liquidation_threshold.check().is_ok());
}

#[test]
fn test_get_ratio_fallback_to_default() {
    let liquidation_threshold = create_sample_liquidation_threshold();

    let res_a_1 = SECP256K1_SIGNATURE_VIRTUAL_BADGE;
    let res_a_4 = GLOBAL_CALLER_VIRTUAL_BADGE;
    let asset_type_1 = 0;
    let asset_type_4 = 3;

    let ratio = liquidation_threshold.get_ratio(
        //
        res_a_1,
        asset_type_1,
        res_a_4,
        asset_type_4,
    );
    assert_eq!(ratio, dec!(0));
}

#[test]
fn test_get_ratio_identical_resource() {
    let liquidation_threshold = create_sample_liquidation_threshold();

    let res_a_1 = SECP256K1_SIGNATURE_VIRTUAL_BADGE;
    let asset_type_1 = 0;
    let asset_type_4 = 3;

    let ratio = liquidation_threshold.get_ratio(
        //
        res_a_1,
        asset_type_1,
        res_a_1,
        asset_type_4,
    );
    assert_eq!(ratio, dec!(0.8));
}

#[test]
fn test_get_ratio_identical_asset_type() {
    let liquidation_threshold = create_sample_liquidation_threshold();

    let res_a_1 = SECP256K1_SIGNATURE_VIRTUAL_BADGE;
    let res_a_2 = ED25519_SIGNATURE_VIRTUAL_BADGE;
    let asset_type_1 = 0;

    let ratio = liquidation_threshold.get_ratio(
        //
        res_a_1,
        asset_type_1,
        res_a_2,
        asset_type_1,
    );
    assert_eq!(ratio, dec!(0.1));
}

#[test]
fn test_get_ratio_resource_1() {
    let liquidation_threshold = create_sample_liquidation_threshold();

    let res_a_1 = SECP256K1_SIGNATURE_VIRTUAL_BADGE;
    let res_a_2 = ED25519_SIGNATURE_VIRTUAL_BADGE;
    let asset_type_1 = 0;
    let asset_type_2 = 1;

    let ratio = liquidation_threshold.get_ratio(
        //
        res_a_2,
        asset_type_2,
        res_a_1,
        asset_type_1,
    );
    assert_eq!(ratio, dec!(0.2));
}

#[test]
fn test_get_ratio_asset_type_1() {
    let liquidation_threshold = create_sample_liquidation_threshold();

    let res_a_1 = SECP256K1_SIGNATURE_VIRTUAL_BADGE;
    let res_a_4 = GLOBAL_CALLER_VIRTUAL_BADGE;
    let asset_type_1 = 0;
    let asset_type_2 = 1;

    let ratio = liquidation_threshold.get_ratio(
        //
        res_a_1,
        asset_type_2,
        res_a_4,
        asset_type_1,
    );
    assert_eq!(ratio, dec!(0.5));
}
