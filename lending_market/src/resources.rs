use crate::modules::cdp_data::*;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AdminBadgeData {}

#[derive(ScryptoSbor)]
pub struct BatchFlashloanItem {
    pub loan_amount: Decimal,
    pub fee_amount: Decimal,
    pub paid_back: bool,
}

#[derive(ScryptoSbor)]
pub struct LiquidationTerm {
    pub cdp_id: NonFungibleLocalId,
    pub payement_value: Decimal,
}
#[derive(ScryptoSbor)]
pub enum TransientResDataType {
    BatchFlashloanItem(IndexMap<ResourceAddress, BatchFlashloanItem>),
    LiquidationTerm(LiquidationTerm),
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct TransientResData {
    pub data: TransientResDataType,
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
                metadata_locker_updater => owner_rule;
            }
        ))
        .with_address(address_reservation)
        .mint_initial_supply([
            (1u64.into(), AdminBadgeData {}),
            (2u64.into(), AdminBadgeData {}),
            (3u64.into(), AdminBadgeData {}),
            (4u64.into(), AdminBadgeData {}),
            (5u64.into(), AdminBadgeData {}),
            (6u64.into(), AdminBadgeData {}),
            (7u64.into(), AdminBadgeData {}),
        ])
}

pub fn create_reserve_collector_badge(owner_rule: AccessRule) -> NonFungibleBucket {
    ResourceBuilder::new_integer_non_fungible::<AdminBadgeData>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule;
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
                metadata_locker_updater => owner_rule;
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
          non_fungible_data_updater => component_rule;
          non_fungible_data_updater_updater => rule!(deny_all);
        })
        .create_with_no_initial_supply()
}

pub fn create_transient_res_manager(
    owner_rule: AccessRule,
    component_rule: AccessRule,
) -> ResourceManager {
    ResourceBuilder::new_ruid_non_fungible::<TransientResData>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule;
            }
        ))
        .mint_roles(mint_roles! {
            minter => component_rule.clone();
            minter_updater => rule!(deny_all);
        })
        .burn_roles(burn_roles! {
            burner => component_rule;
            burner_updater => rule!(deny_all);
        })
        .deposit_roles(deposit_roles! {
            depositor => rule!(deny_all);
            depositor_updater => rule!(deny_all);
        })
        .create_with_no_initial_supply()
}
