// Rev:  1.0.0
// MIT License
// Copyright (c) 2023 @WeftFinance

use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct FlashloanTerm {
    pub loan_amount: Decimal,
    pub fee_amount: Decimal,
}

#[derive(ScryptoSbor, PartialEq)]
pub enum WithdrawType {
    ForTemporaryUse,
    LiquidityWithdrawal,
}

#[derive(ScryptoSbor, PartialEq)]
pub enum DepositType {
    FromTemporaryUse,
    LiquiditySupply,
}

pub fn assert_fungible_res_address(address: ResourceAddress, message: Option<String>) {
    assert!(
        ResourceManager::from_address(address)
            .resource_type()
            .is_fungible(),
        "{}",
        message.unwrap_or("Resource must be fungible".into())
    );
}

pub fn assert_non_fungible_res_address(address: ResourceAddress, message: Option<String>) {
    assert!(
        !ResourceManager::from_address(address)
            .resource_type()
            .is_fungible(),
        "{}",
        message.unwrap_or("Resource must be non fungible".into())
    );
}

#[blueprint]
pub mod single_resource_pool {

    enable_method_auth! {
        roles {
            admin => updatable_by: [];
            can_contribute => updatable_by: [];
            can_redeem => updatable_by: [];
        },
        methods {

            protected_deposit => restrict_to :[admin];
            protected_withdraw => restrict_to :[admin];

            decrease_external_liquidity => restrict_to :[admin];
            increase_external_liquidity => restrict_to :[admin];

            contribute => restrict_to :[can_contribute];
            redeem  => restrict_to :[can_redeem];

            get_pool_unit_ratio => PUBLIC;
            get_pool_unit_supply => PUBLIC;
            get_pooled_amount => PUBLIC;

        }
    }

    pub struct SingleResourcePool {
        /// Vault containing the pooled token
        liquidity: Vault,

        /// Amount taken from the pool and not yet returned
        external_liquidity_amount: Decimal,

        /// Pool unit fungible resource manager
        pool_unit_res_manager: ResourceManager,

        /// Ratio between the pool unit and the pooled token
        unit_to_asset_ratio: PreciseDecimal,
    }

    impl SingleResourcePool {
        pub fn instantiate_locally(
            pool_res_address: ResourceAddress,
            owner_role: OwnerRole,
            component_rule: AccessRule,
        ) -> (Owned<SingleResourcePool>, ResourceAddress) {
            /* CHECK INPUTS */
            assert_fungible_res_address(pool_res_address, None);

            let pool_unit_res_manager = ResourceBuilder::new_fungible(owner_role)
                .mint_roles(mint_roles! {
                    minter => component_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner => component_rule;
                    burner_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            let pool_component = Self {
                liquidity: Vault::new(pool_res_address),
                pool_unit_res_manager,
                external_liquidity_amount: 0.into(),
                unit_to_asset_ratio: 1.into(),
            }
            .instantiate();

            (pool_component, pool_unit_res_manager.address())
        }

        pub fn instantiate(
            pool_res_address: ResourceAddress,
            owner_role: OwnerRole,
            admin_rule: AccessRule,
            contribute_rule: AccessRule,
            redeem_rule: AccessRule,
        ) -> (Global<SingleResourcePool>, ResourceAddress) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(SingleResourcePool::blueprint_id());

            let component_rule = rule!(require(global_caller(component_address)));

            let (owned_pool_component, pool_unit_res_address) =
                SingleResourcePool::instantiate_locally(
                    pool_res_address,
                    owner_role.clone(),
                    component_rule,
                );

            let pool_component = owned_pool_component
                .prepare_to_globalize(owner_role)
                .roles(roles!(
                    admin => admin_rule;
                    can_contribute => contribute_rule;
                    can_redeem => redeem_rule;
                ))
                .with_address(address_reservation)
                .globalize();

            (pool_component, pool_unit_res_address)
        }

        pub fn get_pool_unit_ratio(&self) -> PreciseDecimal {
            self.unit_to_asset_ratio
        }

        pub fn get_pool_unit_supply(&self) -> Decimal {
            self.pool_unit_res_manager.total_supply().unwrap_or(dec!(0))
        }

        pub fn get_pooled_amount(&self) -> (Decimal, Decimal) {
            (self.liquidity.amount(), self.external_liquidity_amount)
        }

        // Handle request to increase liquidity.
        // Add liquidity to the pool and get pool units back
        pub fn contribute(&mut self, assets: Bucket) -> Bucket {
            /* CHECK INPUT */
            assert!(
                assets.resource_address() == self.liquidity.resource_address(),
                "Pool resource address mismatch"
            );

            let unit_amount =
                (assets.amount() * self.unit_to_asset_ratio) //
                    .checked_truncate(RoundingMode::ToNearestMidpointToEven)
                    .expect("Error while calculating unit amount to mint");

            self.liquidity.put(assets);

            self.pool_unit_res_manager.mint(unit_amount)
        }

        // Handle request to decrease liquidity.
        // Remove liquidity from the pool and and burn corresponding pool units
        pub fn redeem(&mut self, pool_units: Bucket) -> Bucket {
            /* INPUT CHECK */
            assert!(
                pool_units.resource_address() == self.pool_unit_res_manager.address(),
                "Pool unit resource address mismatch"
            );

            let amount = (pool_units.amount() / self.unit_to_asset_ratio) //
                .checked_truncate(RoundingMode::ToNearestMidpointToEven)
                .expect("Error while calculating amount to withdraw");

            self.pool_unit_res_manager.burn(pool_units);

            assert!(
                amount <= self.liquidity.amount(),
                "Not enough liquidity to withdraw this amount"
            );

            self.liquidity.take_advanced(
                amount,
                WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
            )
        }

        pub fn protected_withdraw(
            &mut self,
            amount: Decimal,
            withdraw_type: WithdrawType,
            withdraw_strategy: WithdrawStrategy,
        ) -> Bucket {
            /* INPUT CHECK */
            assert!(amount >= 0.into(), "Withdraw amount must not be negative!");

            let assets = self.liquidity.take_advanced(amount, withdraw_strategy);

            if withdraw_type == WithdrawType::ForTemporaryUse {
                self.external_liquidity_amount += amount;
            } else {
                self.unit_to_asset_ratio = self._get_unit_to_asset_ratio();
            }

            assets
        }

        pub fn protected_deposit(&mut self, assets: Bucket, deposit_type: DepositType) {
            /* INPUT CHECK */
            assert_fungible_res_address(assets.resource_address(), None);

            let amount = assets.amount();
            self.liquidity.put(assets);

            if deposit_type == DepositType::FromTemporaryUse {
                assert!(
                    amount <= self.external_liquidity_amount,
                    "Provided amount is greater than the external liquidity amount!"
                );
                self.external_liquidity_amount -= amount;
            } else {
                self.unit_to_asset_ratio = self._get_unit_to_asset_ratio();
            }
        }

        pub fn increase_external_liquidity(&mut self, amount: Decimal) {
            assert!(
                amount >= 0.into(),
                "External liquidity amount must not be negative!"
            );

            self.external_liquidity_amount += amount;

            self.unit_to_asset_ratio = self._get_unit_to_asset_ratio();
        }

        pub fn decrease_external_liquidity(&mut self, amount: Decimal) {
            /* INPUT CHECK */
            assert!(
                amount >= 0.into(),
                "External liquidity amount must not be negative!"
            );
            assert!(
                amount <= self.external_liquidity_amount,
                "Provided amount is greater than the external liquidity amount!"
            );

            self.external_liquidity_amount -= amount;

            self.unit_to_asset_ratio = self._get_unit_to_asset_ratio();
        }

        /* PRIVATE UTILITY METHODS */

        fn _get_unit_to_asset_ratio(&mut self) -> PreciseDecimal {
            let total_liquidity_amount = self.liquidity.amount() + self.external_liquidity_amount;

            let total_supply = self.pool_unit_res_manager.total_supply().unwrap_or(dec!(0));

            if total_liquidity_amount != 0.into() {
                PreciseDecimal::from(total_supply) / PreciseDecimal::from(total_liquidity_amount)
            } else {
                1.into()
            }
        }
    }
}
