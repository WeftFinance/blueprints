use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
    pub timestamp: i64,
    pub price: Decimal,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct UnstakingReceiptData {
    pub pool_unit_amount: Decimal,
    pub created_at: i64,
    pub claimable_at: i64,
    pub expires_at: i64,
}

#[derive(ScryptoSbor, PartialEq)]
enum StakeResWithdrawalUse {
    FeesConversion,
    Refinance(u8, NonFungibleLocalId),
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct StakeResWithdrawalData {
    pub value: Decimal,
    usage: StakeResWithdrawalUse,
}

#[derive(ScryptoSbor, PartialEq)]
pub enum WithdrawType {
    ForTemporaryUse,
    LiquidityWithdrawal,
}

#[derive(ScryptoSbor, PartialEq)]
pub enum DepositType {
    FromTemporaryUse,
    LiquidityAddition,
}

#[blueprint]
pub mod staking_pool {

    extern_blueprint!(
        "package_tdx_2_1p4l8s3qymr20yr7hchwex582z3lmm37p8g56qzvtqerm3av8rtn0ue",  // stokenet
        // "package_sim1pkwaf2l9zkmake5h924229n44wp5pgckmpn0lvtucwers56awywems", // resim
        // "package_sim1p40gjy9kwhn9fjwf9jur0axx72f7c36l6tx3z3vzefp0ytczcql99n", // testing
        SingleResourcePool {

            fn instantiate(
                pool_res_address: ResourceAddress,
                owner_role: OwnerRole,
                admin_rule: AccessRule,
                contribute_rule: AccessRule,
                redeem_rule: AccessRule,
            ) -> (Global<SingleResourcePool>, ResourceAddress);

            fn contribute(&self, assets: Bucket) -> Bucket;

            fn redeem(&self, pool_units: Bucket) -> Bucket;

            fn protected_deposit(&mut self, assets: Bucket, deposit_type: DepositType);

            fn protected_withdraw(
                &self,
                amount: Decimal,
                withdraw_type: WithdrawType,
                withdraw_strategy: WithdrawStrategy
            ) -> Bucket;

            fn increase_external_liquidity(&mut self, amount: Decimal);

            fn get_pool_unit_ratio(&self) -> PreciseDecimal;

            fn get_pooled_amount(&self) -> (Decimal,Decimal);

        }
    );

    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            register_market_component => restrict_to: [admin];

            withdraw_collected_fees => PUBLIC;
            deposit_converted_fees => PUBLIC;

            stake => PUBLIC;
            unstake => PUBLIC;
            claim => PUBLIC;

            start_refinance => PUBLIC;
            end_refinance => PUBLIC;

        }
    }

    pub struct StakingPool {
        ///
        treasury: Global<Account>,

        ///
        asset_pool: Global<SingleResourcePool>,

        ///
        pool_res_address: ResourceAddress,

        ///
        pool_unit_res_address: ResourceAddress,

        ///
        unstaking_pool_unit: Vault,

        ///
        unstaking_receipt_res_manager: ResourceManager,

        ///
        fees_withdrawal_term_res_manager: ResourceManager,

        ///
        market_entries: IndexMap<u8, (Global<AnyComponent>, Bucket)>,

        ///
        price_feed_component: Global<AnyComponent>,

        //* Config * //
        treasury_share: Decimal,

        collection_bonus_rate: Decimal,

        refinance_bonus_rate: Decimal,

        unstake_waiting_time: i64,

        unstake_expiration_time: i64,
    }
    impl StakingPool {
        pub fn instantiate(
            owner_rule: AccessRule,
            pool_res_address: ResourceAddress,
            price_feed_comp_address: ComponentAddress,
        ) -> (Global<StakingPool>, Bucket) {
            //
            let (address_reservation, own_component_address) =
                Runtime::allocate_component_address(StakingPool::blueprint_id());
            let component_rule = rule!(require(global_caller(own_component_address)));

            let unstaking_receipt_res_manager =
                create_unstake_receipt_res_manager(owner_rule.clone(), component_rule.clone());

            let fees_withdrawal_term_res_manager =
                create_fees_withdrawal_term_res_manager(owner_rule.clone(), component_rule.clone());

            let (asset_pool, pool_unit_res_address) = Blueprint::<SingleResourcePool>::instantiate(
                pool_res_address,
                OwnerRole::Fixed(owner_rule.clone()),
                component_rule.clone(),
                component_rule.clone(),
                component_rule,
            );

            let (treasury, treasury_owner_badge) = Blueprint::<Account>::create();

            let comp = Self {
                // * Treasury * //
                treasury,

                // * Runtime * //
                pool_res_address,
                pool_unit_res_address,
                asset_pool,
                unstaking_receipt_res_manager,
                fees_withdrawal_term_res_manager,
                unstaking_pool_unit: Vault::new(pool_unit_res_address),
                market_entries: IndexMap::new(),
                price_feed_component: price_feed_comp_address.into(),

                // * Config * //
                treasury_share: dec!(0.5),
                collection_bonus_rate: dec!(0.03),
                refinance_bonus_rate: dec!(0.03),
                unstake_waiting_time: 7 * 24 * 60 * 60, // in seconds
                unstake_expiration_time: 2 * 24 * 60 * 60, // in seconds
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .roles(roles! {
                admin => owner_rule.clone();
            })
            .metadata(metadata!(
                roles {
                    metadata_setter => owner_rule.clone();
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => owner_rule;
                    metadata_locker_updater => rule!(deny_all);
                }
            ))
            .globalize();

            (comp, treasury_owner_badge)
        }

        // * Admin methods * //

        /// Register lending market component to be able to collect fees from it
        pub fn register_market_component(
            &mut self,
            market_component_address: ComponentAddress,
            fee_collerctor_badge: Bucket,
        ) {
            let len = self.market_entries.len() as u8;
            self.market_entries
                .insert(len, (market_component_address.into(), fee_collerctor_badge));
        }

        // * User methods * //

        /// Collect fees from a registered lending markets
        pub fn withdraw_collected_fees(&mut self, market_comp_index: u8) -> (Vec<Bucket>, Bucket) {
            let result = self._collect_reserve(market_comp_index);

            let mut fees_value = dec!(0);
            let mut assets = Vec::new();

            for (price, asset_bukect) in result {
                fees_value += asset_bukect.amount() * price;
                assets.push(asset_bukect);
            }

            let fee_withdrawal_trensient = self
                .fees_withdrawal_term_res_manager
                .mint_ruid_non_fungible(StakeResWithdrawalData {
                    value: fees_value * (1 - dec!(0.01)),
                    usage: StakeResWithdrawalUse::FeesConversion,
                });

            (assets, fee_withdrawal_trensient)
        }

        /// Deposit converted fees to the pool
        pub fn deposit_converted_fees(
            &mut self,
            mut assets: Bucket,
            fee_withdrawal_trensient: Bucket,
        ) -> Bucket {
            assert!(
                assets.resource_address() == self.pool_res_address
                    && fee_withdrawal_trensient.resource_address()
                        == self.fees_withdrawal_term_res_manager.address(),
                "Invalid assets provided"
            );

            let withdrawal_data: StakeResWithdrawalData = fee_withdrawal_trensient
                .as_non_fungible()
                .non_fungible()
                .data();

            assert!(
                withdrawal_data.usage == StakeResWithdrawalUse::FeesConversion,
                "Invalid withdrawal transient provided"
            );

            let asset_amount = withdrawal_data.value / self.get_price(self.pool_res_address);

            assert!(
                assets.amount() >= asset_amount,
                "Insufficient assets provided"
            );

            self.asset_pool.protected_deposit(
                assets.take_advanced(
                    asset_amount,
                    WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
                ),
                DepositType::LiquidityAddition,
            );

            self.fees_withdrawal_term_res_manager
                .burn(fee_withdrawal_trensient);

            assets
        }

        /// Stake assets to the pool
        pub fn stake(&self, assets: Bucket) -> Bucket {
            self.asset_pool.contribute(assets)
        }

        /// Unstake assets from the pool. Returns unstaking receipt that can be used to claim assets back after unstake waiting time
        pub fn unstake(&mut self, pool_uints: Bucket) -> Bucket {
            let now = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;

            let unstaking_receipt =
                self.unstaking_receipt_res_manager
                    .mint_ruid_non_fungible(UnstakingReceiptData {
                        pool_unit_amount: pool_uints.amount(),
                        created_at: now,
                        claimable_at: now + self.unstake_waiting_time,
                        expires_at: now + self.unstake_waiting_time + self.unstake_expiration_time,
                    });

            self.unstaking_pool_unit.put(pool_uints);

            unstaking_receipt
        }

        /// Claim assets back after unstake waiting time
        pub fn claim(&mut self, unstatking_receipt: Bucket) -> Bucket {
            let unstatking_receipt_data: UnstakingReceiptData =
                unstatking_receipt.as_non_fungible().non_fungible().data();

            assert!(
                unstatking_receipt_data.claimable_at
                    >= Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch,
                "Unstaking receipt is not claimable yet"
            );

            self.unstaking_receipt_res_manager.burn(unstatking_receipt);

            self.asset_pool
                .redeem(self.unstaking_pool_unit.take_advanced(
                    unstatking_receipt_data.pool_unit_amount,
                    WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
                ))
        }

        /// Refinance a LendingMarket defected CDP
        pub fn start_refinance(
            &mut self,
            market_comp_index: u8,
            cdp_id: NonFungibleLocalId,
            amount: Decimal,
        ) -> (Bucket, Bucket) {
            let stacked_resource = self.asset_pool.protected_withdraw(
                amount,
                WithdrawType::ForTemporaryUse,
                WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
            );

            let staked_res_value =
                stacked_resource.amount() * self.get_price(self.pool_res_address);

            let withdrawal_trensient = self
                .fees_withdrawal_term_res_manager
                .mint_ruid_non_fungible(StakeResWithdrawalData {
                    value: staked_res_value * (1 - self.refinance_bonus_rate),
                    usage: StakeResWithdrawalUse::Refinance(market_comp_index, cdp_id),
                });

            (stacked_resource, withdrawal_trensient)
        }

        /// End refinance a LendingMarket defected CDP
        pub fn end_refinance(
            &self,
            payments: Vec<Bucket>,
            withdrawal_trensient: Bucket,
        ) -> Vec<Bucket> {
            assert!(
                withdrawal_trensient.resource_address()
                    == self.fees_withdrawal_term_res_manager.address(),
                "Invalid assets"
            );

            let withdrawal_data: StakeResWithdrawalData =
                withdrawal_trensient.as_non_fungible().non_fungible().data();

            let expected_payment_value = withdrawal_data.value;

            let (market_comp_index, cdp_id) = match withdrawal_data.usage {
                StakeResWithdrawalUse::Refinance(market_comp_index, cdp_id) => {
                    (market_comp_index, cdp_id)
                }
                _ => panic!("Invalid withdrawal transient provided"),
            };

            let (remainder, payment_value) = self._refinance(market_comp_index, cdp_id, payments);

            let payment_check = ((payment_value - expected_payment_value) / payment_value)
                .checked_abs()
                .unwrap()
                <= self.refinance_bonus_rate;

            assert!(
                payment_check,
                "Payment value is not equal to expected payment value"
            );

            self.fees_withdrawal_term_res_manager
                .burn(withdrawal_trensient);

            remainder
        }

        //

        fn get_price(&self, asset: ResourceAddress) -> Decimal {
            self.price_feed_component
                .call_raw::<Option<PriceInfo>>("get_price", scrypto_args!(asset))
                .expect("Price not found")
                .price
        }

        fn _collect_reserve(&self, market_comp_index: u8) -> Vec<(Decimal, Bucket)> {
            let (market_component, collector_badge) = self
                .market_entries
                .get(&market_comp_index)
                .expect("Market component not found. Make sure that you have registered it before");

            let result = collector_badge.as_non_fungible().authorize_with_all(|| {
                market_component
                    .call_raw::<Vec<(Decimal, Bucket)>>("collect_reserve", scrypto_args!())
            });

            result
        }

        fn _refinance(
            &self,
            market_comp_index: u8,
            cdp_id: NonFungibleLocalId,
            payments: Vec<Bucket>,
        ) -> (Vec<Bucket>, Decimal) {
            let (market_component, _) = self
                .market_entries
                .get(&market_comp_index)
                .expect("Market component not found. Make sure that you have registered it before");

            let result = market_component
                .call_raw::<(Vec<Bucket>, Decimal)>("refinance", scrypto_args!(cdp_id, payments));

            result
        }
    }
}

///
///
///
pub fn create_unstake_receipt_res_manager(
    owner_rule: AccessRule,
    component_rule: AccessRule,
) -> ResourceManager {
    ResourceBuilder::new_ruid_non_fungible::<UnstakingReceiptData>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter =>owner_rule.clone();
                metadata_setter_updater => rule!(deny_all);
                metadata_locker => owner_rule;
                metadata_locker_updater => rule!(deny_all);
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
        .create_with_no_initial_supply()
}

///
///
///
fn create_fees_withdrawal_term_res_manager(
    owner_rule: AccessRule,
    component_rule: AccessRule,
) -> ResourceManager {
    ResourceBuilder::new_ruid_non_fungible::<StakeResWithdrawalData>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => rule!(deny_all);
                metadata_locker => owner_rule;
                metadata_locker_updater => rule!(deny_all);
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
