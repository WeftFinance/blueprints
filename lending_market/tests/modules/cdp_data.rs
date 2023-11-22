use lending_market::modules::cdp_data::*;
use scrypto::*;
use scrypto_test::prelude::*;

#[test]
fn test_cdp_type_is_delegator() {
    let delegator_info = DelegateeInfo {
        delegatee_count: 1,
        linked_count: 1,
    };
    let cdp_type = CDPType::Delegator(delegator_info);
    assert!(cdp_type.is_delegator());
}

#[test]
fn test_cdp_type_is_not_delegator() {
    let cdp_type = CDPType::Standard;
    assert!(!cdp_type.is_delegator());
}

#[test]
fn test_cdp_type_is_delegatee() {
    let delegatee_info = DelegatorInfo {
        cdp_id: 1u64.into(),
        delegatee_index: 1,
        max_loan_value: Some(dec!(100)),
        max_loan_value_ratio: Some(dec!(0.5)),
    };
    let cdp_type = CDPType::Delegatee(delegatee_info);
    assert!(cdp_type.is_delegatee());
}

#[test]
fn test_cdp_type_is_not_delegatee() {
    let cdp_type = CDPType::Standard;
    assert!(!cdp_type.is_delegatee());
}

#[test]
fn test_get_collateral_units() {
    let res_address = XRD;
    let mut collaterals = IndexMap::new();
    collaterals.insert(res_address.clone(), dec!(10));
    let cdp_data = CollaterizedDebtPositionData {
        key_image_url: "url".to_string(),
        name: "name".to_string(),
        description: "description".to_string(),
        cdp_type: CDPType::Standard,
        collaterals,
        loans: IndexMap::new(),
        delegatee_loans: IndexMap::new(),
        minted_at: 0,
        updated_at: 0,
    };
    let wrapped_cdp_data = WrappedCDPData {
        cdp_data,
        cdp_id: 1u64.into(),
        cdp_type_updated: false,
        collateral_updated: false,
        loan_updated: false,
        delegatee_loan_updated: false,
    };
    assert_eq!(wrapped_cdp_data.get_collateral_units(res_address), dec!(10));
}

#[test]
fn test_get_loan_unit() {
    let res_address = XRD;

    let mut loans = IndexMap::new();
    loans.insert(res_address.clone(), dec!(10));
    let cdp_data = CollaterizedDebtPositionData {
        key_image_url: "url".to_string(),
        name: "name".to_string(),
        description: "description".to_string(),
        cdp_type: CDPType::Standard,
        collaterals: IndexMap::new(),
        loans,
        delegatee_loans: IndexMap::new(),
        minted_at: 0,
        updated_at: 0,
    };
    let wrapped_cdp_data = WrappedCDPData {
        cdp_data,
        cdp_id: 1u64.into(),
        cdp_type_updated: false,
        collateral_updated: false,
        loan_updated: false,
        delegatee_loan_updated: false,
    };
    assert_eq!(wrapped_cdp_data.get_loan_unit(res_address), dec!(10));
}

#[test]
fn test_increase_delegatee_count() {
    let delegator_info = DelegateeInfo {
        delegatee_count: 0,
        linked_count: 0,
    };
    let mut cdp_type = CDPType::Delegator(delegator_info);
    let mut wrapped_cdp_data = WrappedCDPData {
        cdp_data: CollaterizedDebtPositionData {
            key_image_url: "url".to_string(),
            name: "name".to_string(),
            description: "description".to_string(),
            cdp_type: cdp_type.clone(),
            collaterals: IndexMap::new(),
            loans: IndexMap::new(),
            delegatee_loans: IndexMap::new(),
            minted_at: 0,
            updated_at: 0,
        },
        cdp_id: 1u64.into(),
        cdp_type_updated: false,
        collateral_updated: false,
        loan_updated: false,
        delegatee_loan_updated: false,
    };
    wrapped_cdp_data.increase_delegatee_count().unwrap();

    cdp_type = CDPType::Delegator(DelegateeInfo {
        delegatee_count: 1,
        linked_count: 1,
    });

    assert_eq!(wrapped_cdp_data.cdp_data.cdp_type, cdp_type);
}

#[test]
fn test_decrease_delegatee_count() {
    let delegatee_info = DelegateeInfo {
        delegatee_count: 2,
        linked_count: 2,
    };
    let mut cdp_type = CDPType::Delegator(delegatee_info);
    let mut wrapped_cdp_data = WrappedCDPData {
        cdp_data: CollaterizedDebtPositionData {
            key_image_url: "url".to_string(),
            name: "name".to_string(),
            description: "description".to_string(),
            cdp_type: cdp_type.clone(),
            collaterals: IndexMap::new(),
            loans: IndexMap::new(),
            delegatee_loans: IndexMap::new(),
            minted_at: 0,
            updated_at: 0,
        },
        cdp_id: 1u64.into(),
        cdp_type_updated: false,
        collateral_updated: false,
        loan_updated: false,
        delegatee_loan_updated: false,
    };
    wrapped_cdp_data.decrease_delegatee_count().unwrap();
    cdp_type = CDPType::Delegator(DelegateeInfo {
        delegatee_count: 1,
        linked_count: 2,
    });
    assert_eq!(wrapped_cdp_data.cdp_data.cdp_type, cdp_type);

    wrapped_cdp_data.decrease_delegatee_count().unwrap();
    cdp_type = CDPType::Standard;
    assert_eq!(wrapped_cdp_data.cdp_data.cdp_type, cdp_type);
}

#[test]
fn test_update_collateral() {
    let res_address = XRD;
    let mut wrapped_cdp_data = WrappedCDPData {
        cdp_data: CollaterizedDebtPositionData {
            key_image_url: "url".to_string(),
            name: "name".to_string(),
            description: "description".to_string(),
            cdp_type: CDPType::Standard,
            collaterals: IndexMap::new(),
            loans: IndexMap::new(),
            delegatee_loans: IndexMap::new(),
            minted_at: 0,
            updated_at: 0,
        },
        cdp_id: 1u64.into(),
        cdp_type_updated: false,
        collateral_updated: false,
        loan_updated: false,
        delegatee_loan_updated: false,
    };
    wrapped_cdp_data
        .update_collateral(res_address.clone(), dec!(10))
        .unwrap();
    let mut collaterals = IndexMap::new();
    collaterals.insert(res_address.clone(), dec!(10));

    assert_eq!(wrapped_cdp_data.cdp_data.collaterals, collaterals);
}

#[test]
fn test_update_loan() {
    let res_address = XRD;
    let mut wrapped_cdp_data = WrappedCDPData {
        cdp_data: CollaterizedDebtPositionData {
            key_image_url: "url".to_string(),
            name: "name".to_string(),
            description: "description".to_string(),
            cdp_type: CDPType::Standard,
            collaterals: IndexMap::new(),
            loans: IndexMap::new(),
            delegatee_loans: IndexMap::new(),
            minted_at: 0,
            updated_at: 0,
        },
        cdp_id: 1u64.into(),
        cdp_type_updated: false,
        collateral_updated: false,
        loan_updated: false,
        delegatee_loan_updated: false,
    };
    wrapped_cdp_data
        .update_loan(res_address.clone(), dec!(10))
        .unwrap();
    let mut loans = IndexMap::new();
    loans.insert(res_address.clone(), dec!(10));
    assert_eq!(wrapped_cdp_data.cdp_data.loans, loans);
}

#[test]
fn test_update_delegatee_loan() {
    let res_address = XRD;
    let mut wrapped_cdp_data = WrappedCDPData {
        cdp_data: CollaterizedDebtPositionData {
            key_image_url: "url".to_string(),
            name: "name".to_string(),
            description: "description".to_string(),
            cdp_type: CDPType::Standard,
            collaterals: IndexMap::new(),
            loans: IndexMap::new(),
            delegatee_loans: IndexMap::new(),
            minted_at: 0,
            updated_at: 0,
        },
        cdp_id: 1u64.into(),
        cdp_type_updated: false,
        collateral_updated: false,
        loan_updated: false,
        delegatee_loan_updated: false,
    };
    wrapped_cdp_data
        .update_delegatee_loan(res_address.clone(), dec!(10))
        .unwrap();
    let mut delegatee_loans = IndexMap::new();
    delegatee_loans.insert(res_address.clone(), dec!(10));
    assert_eq!(wrapped_cdp_data.cdp_data.delegatee_loans, delegatee_loans);
}
