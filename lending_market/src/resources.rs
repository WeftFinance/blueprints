use crate::modules::cdp_data::*;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AdminBadgeData {}

#[derive(ScryptoSbor)]
pub struct BatchFlashloanItem {
    pub loan_amount: Decimal,
    pub fee_amount: Decimal,
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct BatchFlashloanTerm {
    pub terms: IndexMap<ResourceAddress, BatchFlashloanItem>,
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct LiquidationTerm {
    pub payement_value: Decimal,
}

pub fn create_admin_badge(
    owner_rule: AccessRule,
    address_reservation: GlobalAddressReservation,
) -> NonFungibleBucket {
    ResourceBuilder::new_integer_non_fungible::<AdminBadgeData>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule.clone();
            }
        ))
        .with_address(address_reservation)
        .mint_initial_supply([(1u64.into(), AdminBadgeData {})])
}

pub fn create_reserve_collector_badge(owner_rule: AccessRule) -> NonFungibleBucket {
    ResourceBuilder::new_integer_non_fungible::<AdminBadgeData>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule.clone();
            }
        ))
        .mint_initial_supply([(1u64.into(), AdminBadgeData {})])
}

pub fn create_cdp_res_manager(
    owner_rule: AccessRule,
    component_rule: AccessRule,
) -> ResourceManager {
    ResourceBuilder::new_integer_non_fungible::<CollaterizedDebtPositionData>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule.clone();
            }
        ))
        .mint_roles(mint_roles! {
          minter => component_rule.clone();
          minter_updater => rule!(deny_all);
        })
        .burn_roles(burn_roles! {
          burner => component_rule.clone();
          burner_updater => rule!(deny_all);
        })
        .non_fungible_data_update_roles(non_fungible_data_update_roles! {
          non_fungible_data_updater => component_rule.clone();
          non_fungible_data_updater_updater => rule!(deny_all);
        })
        .create_with_no_initial_supply()
}

pub fn create_batch_flashloan_term_res_manager(
    owner_rule: AccessRule,
    component_rule: AccessRule,
) -> ResourceManager {
    ResourceBuilder::new_ruid_non_fungible::<BatchFlashloanTerm>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule.clone();
            }
        ))
        .mint_roles(mint_roles! {
            minter => component_rule.clone();
            minter_updater => rule!(deny_all);
        })
        .burn_roles(burn_roles! {
            burner => component_rule.clone();
            burner_updater => rule!(deny_all);
        })
        .deposit_roles(deposit_roles! {
            depositor => rule!(deny_all);
            depositor_updater => rule!(deny_all);
        })
        .create_with_no_initial_supply()
}

pub fn create_liquidation_term_res_manager(
    owner_rule: AccessRule,
    component_rule: AccessRule,
) -> ResourceManager {
    ResourceBuilder::new_ruid_non_fungible::<BatchFlashloanTerm>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule.clone();
            }
        ))
        .mint_roles(mint_roles! {
            minter => component_rule.clone();
            minter_updater => rule!(deny_all);
        })
        .burn_roles(burn_roles! {
            burner => component_rule.clone();
            burner_updater => rule!(deny_all);
        })
        .deposit_roles(deposit_roles! {
            depositor => rule!(deny_all);
            depositor_updater => rule!(deny_all);
        })
        .create_with_no_initial_supply()
}
