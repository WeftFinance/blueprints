use crate::modules::{
    cdp_data::*, cdp_health_checker::*, interest_strategy::*, liquidation_threshold::*,
    market_config::*, operation_status::*, pool_config::*, pool_state::*, utils::*,
};
use crate::resources::*;
use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub enum UpdateCDPInput {
    KeyImageURL(String),
    Name(String),
    Description(String),
}

#[derive(ScryptoSbor)]
pub enum UpdateDelegateeCDPnput {
    MaxLoanValue(Decimal),
    MaxToLoanValue(Decimal),
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct CDPUpdatedEvent {
    pub cdp_id: NonFungibleLocalId,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct CDPLiquidatedEvent {
    pub cdp_id: NonFungibleLocalId,
    pub delegator_cdp_id: Option<NonFungibleLocalId>,
}

#[derive(ScryptoSbor)]
pub enum LendingPoolUpdatedEventType {
    Deposit,
    Loan,
    Collateral,
    Interest,
    Price,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct LendingPoolUpdatedEvent {
    pub pool_res_address: ResourceAddress,
    pub event_type: LendingPoolUpdatedEventType,
}

#[blueprint]
#[events(CDPUpdatedEvent, CDPLiquidatedEvent, LendingPoolUpdatedEvent)]
mod lending_market {

    extern_blueprint!(
        // "package_tdx_2_1p4wnzxlrcv9s6hsy7fdv8td06up4wzwe5vjpmw8f8jgyj4z6vhqnl5",  // stokenet
        // "package_sim1pkwaf2l9zkmake5h924229n44wp5pgckmpn0lvtucwers56awywems", // resim
         "package_sim1ph8fqgwl6sdmlxxv06sf2sgk3jp9l5vrrc2enpqm5hx686auz0d9k5", // testing
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
            moderator => updatable_by: [];
            reserve_collector => updatable_by: [];
        },

        methods {

            /* Admin methods */

            create_lending_pool => restrict_to: [admin];

            set_price_feed => restrict_to: [admin];

            update_market_config => restrict_to: [admin];
            update_pool_config => restrict_to: [admin];
            update_liquidation_threshold => restrict_to: [admin];
            set_interest_strategy => restrict_to: [admin];

            admin_update_operating_status => restrict_to: [admin];

            update_operating_status => restrict_to: [admin,moderator];

            update_pool_state => PUBLIC;

            /* Reserve Collector methods*/

            collect_reserve => restrict_to: [reserve_collector];

            /* User methods */

            // CDP Management methods

            create_cdp => PUBLIC;
            create_delegatee_cdp => PUBLIC;

            link_cdp => PUBLIC;
            unlink_cdp => PUBLIC;

            update_cdp => PUBLIC;
            update_delegatee_cdp => PUBLIC;

            // Flashloan methods

            take_batch_flashloan => PUBLIC;
            repay_batch_flashloan => PUBLIC;

            // Lending and Borrowing methods

            contribute => PUBLIC;
            redeem => PUBLIC;

            add_collateral => PUBLIC;
            remove_collateral => PUBLIC;
            borrow => PUBLIC;
            repay => PUBLIC;

            // Liquidation methods

            refinance => PUBLIC;
            start_liquidation => PUBLIC;
            end_liquidation => PUBLIC;
            fast_liquidation => PUBLIC;
        }

    }

    struct LendingMarket {
        /// Save the admin rule for lending pool creation
        admin_rule: AccessRule,

        ///
        fee_subsid_reserve: Vault,

        ///
        cdp_res_manager: ResourceManager,

        ///
        cdp_counter: u64,

        /// Current lending market component address
        market_component_address: ComponentAddress,

        ///
        pool_unit_refs: IndexMap<ResourceAddress, ResourceAddress>,

        ///
        revers_pool_unit_refs: IndexMap<ResourceAddress, ResourceAddress>,

        ///
        listed_assets: IndexSet<ResourceAddress>,

        ///
        pool_states: KeyValueStore<ResourceAddress, LendingPoolState>,

        ///
        batch_flashloan_term_res_manager: ResourceManager,

        ///
        liquidation_term_res_manager: ResourceManager,

        ///
        operating_status: OperatingStatus,

        ///
        market_config: MarketConfig,

        ///
        delegatee_cdp_ids: KeyValueStore<(NonFungibleLocalId, u64), NonFungibleLocalId>,
    }

    impl LendingMarket {
        pub fn instantiate(market_config: MarketConfig) -> (NonFungibleBucket, NonFungibleBucket) {
            // Check inputs
            market_config.check().expect("Invalid market config");

            // Get address reservation for the lending market component
            let (market_component_address_reservation, market_component_address) =
                Runtime::allocate_component_address(LendingMarket::blueprint_id());
            let component_rule = rule!(require(global_caller(market_component_address)));

            // * Create admin badge * //

            // Get address reservation for the admin badge resource address
            let (admin_badge_address_reservation, admin_badge_address) =
                Runtime::allocate_non_fungible_address();

            // Admin will be able to create lending pools, update pool configurations and update operating status
            let admin_rule = rule!(require_amount(dec!(4), admin_badge_address));

            // Moderator will be able to update operating status if the last update was not done by an admin
            let modarator_rule = rule!(require_amount(dec!(2), admin_badge_address));

            let admin_badge =
                create_admin_badge(admin_rule.clone(), admin_badge_address_reservation);

            // * Create reserve collector badge * //

            let reserve_collector_badge = create_reserve_collector_badge(admin_rule.clone());
            let reserve_collector_rule = rule!(require(reserve_collector_badge.resource_address()));

            // * Create CDP resource manager * //
            let cdp_res_manager =
                create_cdp_res_manager(admin_rule.clone(), component_rule.clone());

            // * Create batch flashloan term resource manager * //
            let batch_flashloan_term_res_manager =
                create_batch_flashloan_term_res_manager(admin_rule.clone(), component_rule.clone());

            // * Create liquidation term resource manager * //
            let liquidation_term_res_manager =
                create_liquidation_term_res_manager(admin_rule.clone(), component_rule);

            // *  Instantiate our component with the previously created resources and addresses * //
            Self {
                market_component_address,
                cdp_res_manager,
                admin_rule: admin_rule.clone(),
                cdp_counter: 0,
                fee_subsid_reserve: Vault::new(XRD),
                batch_flashloan_term_res_manager,
                liquidation_term_res_manager,
                pool_unit_refs: IndexMap::new(),
                revers_pool_unit_refs: IndexMap::new(),
                pool_states: KeyValueStore::new(),
                listed_assets: IndexSet::new(),
                operating_status: OperatingStatus::new(),
                market_config,
                delegatee_cdp_ids: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(market_component_address_reservation)
            .roles(roles! {
                admin => admin_rule.clone();
                moderator => modarator_rule;
                reserve_collector => reserve_collector_rule;
            })
            .metadata(metadata!(
                roles {
                    metadata_setter => admin_rule.clone();
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => admin_rule;
                    metadata_locker_updater => rule!(deny_all);
                }
            ))
            .globalize();

            (admin_badge, reserve_collector_badge)
        }

        /*
        POOL MANAGEMENT METHODS
        */

        pub fn create_lending_pool(
            &mut self,
            price_feed_component: Global<AnyComponent>,
            pool_res_address: ResourceAddress,
            pool_config: PoolConfig,
            interest_strategy_break_points: (Decimal, Vec<ISInputBreakPoint>),
            liquidation_threshold: LiquidationThreshold,
        ) {
            assert!(
                self.listed_assets.get(&pool_res_address).is_none(),
                "The lending pool is already registered"
            );

            let res_manager: ResourceManager = pool_res_address.into();

            let recaller_role = res_manager
                .get_role("recaller")
                .expect("Error getting recaller role");

            let recaller_updater_role = res_manager
                .get_role("recaller_updater")
                .expect("Error getting recaller_updater role");

            assert!(
                recaller_role == AccessRule::DenyAll
                    && recaller_updater_role == AccessRule::DenyAll,
                "Recallable assets are not supported"
            );

            liquidation_threshold
                .check()
                .expect("Invalid liquidation threshold");

            pool_config.check().expect("Invalid pool config");

            let component_rule = rule!(require(global_caller(self.market_component_address)));

            let (pool, pool_unit_res_address) = Blueprint::<SingleResourcePool>::instantiate(
                pool_res_address,
                OwnerRole::None,
                component_rule.clone(),
                component_rule.clone(),
                component_rule,
            );

            let mut interest_strategy = InterestStrategy::new();

            // set_breakpoints will check the validity of the breakpoints
            interest_strategy
                .set_breakpoints(
                    interest_strategy_break_points.0,
                    interest_strategy_break_points.1,
                )
                .expect("Invalid interest strategy breakpoints");

            let last_price_info =
                get_price(price_feed_component, pool_res_address).expect("Price not found");

            let pool_state = LendingPoolState {
                pool,
                collaterals: Vault::new(pool_unit_res_address),
                reserve: Vault::new(pool_res_address),
                pool_res_address,

                last_price: last_price_info.price,

                price_updated_at: Clock::current_time(TimePrecision::Minute)
                    .seconds_since_unix_epoch,

                total_loan: 0.into(),
                total_loan_unit: 0.into(),
                interest_rate: 0.into(),
                interest_updated_at: Clock::current_time(TimePrecision::Minute)
                    .seconds_since_unix_epoch,

                price_feed_comp: price_feed_component,
                interest_strategy,
                liquidation_threshold,
                pool_config,
                operating_status: OperatingStatus::new(),
            };

            //
            self.pool_states.insert(pool_res_address, pool_state);

            //
            self.revers_pool_unit_refs
                .insert(pool_unit_res_address, pool_res_address);

            self.pool_unit_refs
                .insert(pool_res_address, pool_unit_res_address);

            self.listed_assets.insert(pool_res_address);
        }

        // Collect reserve retention from all pools
        pub fn collect_reserve(&mut self) -> Vec<(Decimal, Bucket)> {
            let listed_assets = self.listed_assets.clone();

            listed_assets
                .iter()
                .map(|pool_res_address| {
                    let mut pool_state = self._get_pool_state(pool_res_address, None);

                    let price = pool_state.last_price;

                    let fee = pool_state.reserve.take_all();

                    (price, fee)
                })
                .collect()
        }

        pub fn set_price_feed(
            &mut self,
            pool_res_address: ResourceAddress,
            price_feed: Global<AnyComponent>,
        ) {
            let mut pool_state = self._get_pool_state(&pool_res_address, None);

            get_price(price_feed, pool_res_address).expect("Price not found");

            pool_state.set_price_feed(price_feed);
        }

        pub fn update_liquidation_threshold(
            &mut self,
            pool_res_address: ResourceAddress,
            value: UpdateLiquidationThresholdInput,
        ) {
            let mut pool_state = self._get_pool_state(&pool_res_address, None);

            pool_state
                .update_liquidation_threshold(value)
                .expect("Invalid liquidation threshold");
        }

        pub fn set_interest_strategy(
            &mut self,
            pool_res_address: ResourceAddress,
            initial_rate: Decimal,
            interest_options_break_points: Vec<ISInputBreakPoint>,
        ) {
            let mut pool_state = self._get_pool_state(&pool_res_address, None);

            pool_state
                .set_interest_strategy(initial_rate, interest_options_break_points)
                .expect("Invalid interest strategy breakpoints");
        }

        pub fn update_market_config(&mut self, value: UpdateMarketConfigInput) {
            self.market_config
                .update(value)
                .expect("Invalid market config");
        }

        pub fn update_pool_config(
            &mut self,
            pool_res_address: ResourceAddress,
            value: UpdatePoolConfigInput,
        ) {
            let mut pool_state = self._get_pool_state(&pool_res_address, None);

            pool_state
                .update_config(value)
                .expect("Invalid pool config");
        }

        pub fn update_pool_state(&mut self, pool_res_address: ResourceAddress) {
            self._get_pool_state(&pool_res_address, None);
        }

        ///
        fn _update_operating_status(
            &mut self,
            value: OperatingService,
            enabled: bool,
            set_by_admin: bool,
            pool_res_address: Option<ResourceAddress>,
        ) -> Result<(), String> {
            match pool_res_address {
                Some(pool_res_address) => {
                    let mut pool_state = self._get_pool_state(&pool_res_address, None);

                    pool_state
                        .operating_status
                        .update(value, enabled, set_by_admin)
                }
                None => self.operating_status.update(value, enabled, set_by_admin),
            }
        }

        // Update the operating status of the lending market or a specific pool
        // Update made by a moderator can be reverted by an admin
        pub fn update_operating_status(
            &mut self,
            value: OperatingService,
            enabled: bool,
            pool_res_address: Option<ResourceAddress>,
        ) {
            self._update_operating_status(value, enabled, false, pool_res_address)
                .expect("Error updating operating status by a moderator")
        }

        // Update the operating status of the lending market or a specific pool with admin flag
        // Update made by an admin will not be reverted by a moderator
        pub fn admin_update_operating_status(
            &mut self,
            value: OperatingService,
            enabled: bool,
            pool_res_address: Option<ResourceAddress>,
        ) {
            self._update_operating_status(value, enabled, true, pool_res_address)
                .expect("Error updating operating status by an admin")
        }

        ///*  CDP CREATION AND MANAGEMENT METHODS * ///

        pub fn create_cdp(
            &mut self,
            name: Option<String>,
            description: Option<String>,
            key_image_url: Option<String>,
            deposits: Vec<Bucket>,
        ) -> Bucket {
            let cdp_id = NonFungibleLocalId::Integer(self._get_new_cdp_id().into());

            let now = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;

            let data = CollaterizedDebtPositionData {
                name: name.unwrap_or("".into()),
                description: description.unwrap_or("".into()),
                key_image_url: key_image_url.unwrap_or("".into()),
                cdp_type: CDPType::Standard,
                collaterals: IndexMap::new(),
                loans: IndexMap::new(),
                delegatee_loans: IndexMap::new(),
                minted_at: now,
                updated_at: now,
            };

            let cdp = self.cdp_res_manager.mint_non_fungible(&cdp_id, data);

            if !deposits.is_empty() {
                self._deposit_internal(cdp_id, deposits);
            }

            cdp
        }

        // Create a new CDP with borrowing power delegated from a Delegator CDP
        pub fn create_delegatee_cdp(
            &mut self,
            delegator_cdp_proof: Proof,
            max_loan_value: Option<Decimal>,
            max_loan_value_ratio: Option<Decimal>,
            name: Option<String>,
            description: Option<String>,
            key_image_url: Option<String>,
        ) -> Bucket {
            //

            assert!(
                max_loan_value_ratio.unwrap_or(0.into()) >= 0.into()
                    && max_loan_value_ratio.unwrap_or(0.into()) <= 1.into(),
                "INVALID_INPUT: Max loan to value ratio must be in the range [0, 1]"
            );

            assert!(
                max_loan_value.unwrap_or(0.into()) >= 0.into(),
                "INVALID_INPUT: Max loan to value must be non-negative"
            );

            //

            let delegator_cdp_id = self._validate_cdp_proof(delegator_cdp_proof);

            let mut delegator_cdp_data =
                WrappedCDPData::new(&self.cdp_res_manager, &delegator_cdp_id);

            assert!(
                !delegator_cdp_data.is_delegatee(),
                "Delegatee CDP can not create delegatee CDP",
            );

            let (_, linked_count) = delegator_cdp_data
                .increase_delegatee_count()
                .expect("Error increasing delegatee count");

            delegator_cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");

            let now = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;

            let new_cdp_id = self._get_new_cdp_id();

            self.delegatee_cdp_ids.insert(
                (delegator_cdp_id.clone(), linked_count),
                NonFungibleLocalId::Integer(new_cdp_id.into()),
            );

            let delegatee_cdp_data = CollaterizedDebtPositionData {
                name: name.unwrap_or("".into()),
                description: description.unwrap_or("".into()),
                key_image_url: key_image_url.unwrap_or("".into()),
                cdp_type: CDPType::Delegatee(DelegatorInfo {
                    cdp_id: delegator_cdp_id,
                    delegatee_index: linked_count,
                    max_loan_value_ratio,
                    max_loan_value,
                }),
                collaterals: IndexMap::new(),
                loans: IndexMap::new(),
                delegatee_loans: IndexMap::new(),
                minted_at: now,
                updated_at: now,
            };

            let delegatee_cdp_id = NonFungibleLocalId::Integer(new_cdp_id.into());
            self.cdp_res_manager
                .mint_non_fungible(&delegatee_cdp_id, delegatee_cdp_data)
        }

        pub fn link_cdp(
            &mut self,
            delegator_cdp_proof: Proof,
            delegatee_cdp_proof: Proof,
            max_loan_value: Option<Decimal>,
            max_loan_value_ratio: Option<Decimal>,
        ) {
            assert!(
                max_loan_value_ratio.unwrap_or(0.into()) >= 0.into()
                    && max_loan_value_ratio.unwrap_or(0.into()) <= 1.into(),
                "INVALID_INPUT: Max loan to value ratio must be in the range [0, 1]"
            );

            assert!(
                max_loan_value.unwrap_or(0.into()) >= 0.into(),
                "INVALID_INPUT: Max loan to value must be non-negative"
            );

            let delegator_cdp_id = self._validate_cdp_proof(delegator_cdp_proof);

            let delegatee_cdp_id = self._validate_cdp_proof(delegatee_cdp_proof);

            let mut delegatee_cdp_data =
                WrappedCDPData::new(&self.cdp_res_manager, &delegatee_cdp_id);

            assert!(
                delegatee_cdp_data.get_type() == CDPType::Standard,
                "Delegatee CDP already linked",
            );

            // CDP with collateral can not be convert to Delagatee CDP for consistency reason.
            // Delagator and delegatee CDPs should have consistent health status
            assert!(
                delegatee_cdp_data.get_data().collaterals.is_empty(),
                "Delegatee CDP already has collateral",
            );

            let mut delegator_cdp_data =
                WrappedCDPData::new(&self.cdp_res_manager, &delegator_cdp_id);

            let (_, linked_count) = delegator_cdp_data
                .increase_delegatee_count()
                .expect("Error increasing delegatee count");

            delegatee_cdp_data.update_cdp_type(CDPType::Delegatee(DelegatorInfo {
                cdp_id: delegator_cdp_id.clone(),
                delegatee_index: linked_count,
                max_loan_value_ratio,
                max_loan_value,
            }));

            CDPHealthChecker::new(
                &delegatee_cdp_data,
                Some(&delegator_cdp_data),
                &mut self.pool_states,
            )
            .check_cdp()
            .expect("Error checking CDP");

            self.delegatee_cdp_ids
                .insert((delegator_cdp_id, linked_count), delegatee_cdp_id);

            delegator_cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");
            delegatee_cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");
        }

        pub fn unlink_cdp(
            &mut self,
            delegator_cdp_proof: Proof,
            delegatee_cdp_id: NonFungibleLocalId,
        ) {
            let delegator_cdp_id = self._validate_cdp_proof(delegator_cdp_proof);

            let mut delegator_cdp_data =
                WrappedCDPData::new(&self.cdp_res_manager, &delegator_cdp_id);

            let mut delegatee_cdp_data =
                WrappedCDPData::new(&self.cdp_res_manager, &delegatee_cdp_id);

            assert!(
                delegatee_cdp_data
                    .get_delegator_id()
                    .expect("Error getting delegator_id")
                    == delegator_cdp_id,
                "Delegatee CDP not linked to provided delegator CDP",
            );

            if let CDPType::Delegatee(delegatee_cdp_data) = delegatee_cdp_data.get_type() {
                self.delegatee_cdp_ids
                    .remove(&(delegator_cdp_id, delegatee_cdp_data.delegatee_index));
            }

            delegatee_cdp_data.update_cdp_type(CDPType::Standard);

            CDPHealthChecker::new(&delegatee_cdp_data, None, &mut self.pool_states)
                .check_cdp()
                .expect("Error checking CDP");

            delegator_cdp_data
                .decrease_delegatee_count()
                .expect("Error decreasing delegatee count");

            delegatee_cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");

            delegator_cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");
        }

        pub fn update_cdp(&mut self, cdp_proof: Proof, value: UpdateCDPInput) {
            let cdp_id = self._validate_cdp_proof(cdp_proof);

            match value {
                UpdateCDPInput::KeyImageURL(key_image_url) => {
                    self.cdp_res_manager.update_non_fungible_data(
                        &cdp_id,
                        "key_image_url",
                        key_image_url,
                    );
                }
                UpdateCDPInput::Name(name) => {
                    self.cdp_res_manager
                        .update_non_fungible_data(&cdp_id, "name", name);
                }
                UpdateCDPInput::Description(description) => {
                    self.cdp_res_manager.update_non_fungible_data(
                        &cdp_id,
                        "description",
                        description,
                    );
                }
            }

            self.cdp_res_manager.update_non_fungible_data(
                &cdp_id,
                "updated_at",
                Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch,
            );
        }

        pub fn update_delegatee_cdp(
            &mut self,
            delegator_cdp_proof: Proof,
            delegatee_cdp_id: NonFungibleLocalId,
            max_loan_value: Option<Decimal>,
            max_loan_value_ratio: Option<Decimal>,
        ) {
            assert!(
                max_loan_value_ratio.unwrap_or(0.into()) >= 0.into()
                    && max_loan_value_ratio.unwrap_or(0.into()) <= 1.into(),
                "INVALID_INPUT: Max loan to value ratio must be in the range [0, 1]"
            );

            assert!(
                max_loan_value.unwrap_or(0.into()) >= 0.into(),
                "INVALID_INPUT: Max loan to value must be non-negative"
            );

            let delegator_cdp_id = self._validate_cdp_proof(delegator_cdp_proof);

            let mut delegatee_cdp_data =
                WrappedCDPData::new(&self.cdp_res_manager, &delegatee_cdp_id);

            assert!(
                delegatee_cdp_data
                    .get_delegator_id()
                    .expect("Error getting delegator_id")
                    == delegator_cdp_id,
                "Delegatee CDP not linked to provided delegator CDP",
            );

            delegatee_cdp_data
                .update_delegatee_info(max_loan_value, max_loan_value_ratio)
                .expect("Error updating delegatee info");

            delegatee_cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");
        }

        // / * Flashloan methods * ///

        pub fn take_batch_flashloan(
            &mut self,
            loan_amounts: IndexMap<ResourceAddress, Decimal>,
        ) -> (Vec<Bucket>, Bucket) {
            self.check_operating_status(OperatingService::Flashloan);

            let mut loans: Vec<Bucket> = Vec::new();
            let mut terms: IndexMap<ResourceAddress, BatchFlashloanItem> = IndexMap::new();

            for (pool_res_address, amount) in loan_amounts.iter() {
                assert!(
                    amount >= &Decimal::ZERO,
                    "INVALID_INPUT: borrowed amount must be greater than zero"
                );

                let pool_state = self
                    .pool_states
                    .get_mut(pool_res_address)
                    .expect("Pool state not found for provided resource");

                pool_state
                    .check_operating_status(OperatingService::Flashloan)
                    .expect("Flashloan is not enabled for the pool");

                let fee_amount = (*amount) * pool_state.pool_config.flashloan_fee_rate;

                let loan_term = BatchFlashloanItem {
                    fee_amount,
                    loan_amount: *amount,
                };

                let loan = pool_state.pool.protected_withdraw(
                    *amount,
                    WithdrawType::TemporaryUse,
                    WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
                );

                loans.push(loan);
                terms.insert(*pool_res_address, loan_term);
            }

            (
                loans,
                self.batch_flashloan_term_res_manager
                    .mint_ruid_non_fungible(BatchFlashloanTerm { terms }),
            )
        }

        pub fn repay_batch_flashloan(
            &mut self,
            payments: Vec<Bucket>,
            batch_loan_term: Bucket,
        ) -> Vec<Bucket> {
            let mut remainders: Vec<Bucket> = Vec::new();

            let batch_loan_term_data: BatchFlashloanTerm =
                batch_loan_term.as_non_fungible().non_fungible().data();

            for mut payment in payments {
                let pool_res_address = payment.resource_address();

                let loan_term = batch_loan_term_data
                    .terms
                    .get(&pool_res_address)
                    .expect("flash loan term not found for provided resource");

                let due_amount = loan_term.fee_amount + loan_term.loan_amount;

                assert!(
                    payment.amount() >= (due_amount),
                    "Insufficient repayment given for your loan!"
                );

                let mut pool_state = self
                    .pool_states
                    .get_mut(&pool_res_address)
                    .expect("Pool state not found for provided resource");

                pool_state.pool.protected_deposit(
                    payment.take_advanced(
                        loan_term.loan_amount,
                        WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
                    ),
                    DepositType::FromTemporaryUse,
                );

                let protocol_fee_amount =
                    loan_term.fee_amount * pool_state.pool_config.protocol_flashloan_fee_rate;

                let lp_fee_amount = loan_term.fee_amount - protocol_fee_amount;

                pool_state.pool.protected_deposit(
                    payment.take_advanced(
                        lp_fee_amount,
                        WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
                    ),
                    DepositType::LiquiditySupply,
                );

                pool_state.reserve.put(payment.take_advanced(
                    protocol_fee_amount,
                    WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
                ));

                remainders.push(payment);
            }

            self.batch_flashloan_term_res_manager.burn(batch_loan_term);

            remainders
        }

        //* Lending and Borrowing methods * //

        pub fn contribute(&mut self, assets: Bucket) -> Bucket {
            self.check_operating_status(OperatingService::Contribute);

            let pool_state = self._get_pool_state(
                &assets.resource_address(),
                Some(OperatingService::Contribute),
            );

            pool_state
                .contribute_proxy(assets)
                .expect("Error contributing to pool")
        }

        pub fn redeem(&mut self, pool_units: Bucket) -> Bucket {
            self.check_operating_status(OperatingService::Redeem);

            let pool_res_address = *self
                .revers_pool_unit_refs
                .get(&pool_units.resource_address())
                .expect("Pool unit not found");

            self._get_pool_state(&pool_res_address, Some(OperatingService::Redeem))
                .redeem_proxy(pool_units)
        }

        pub fn add_collateral(&mut self, cdp_proof: Proof, deposits: Vec<Bucket>) {
            let cdp_id = self._validate_cdp_proof(cdp_proof);

            self._deposit_internal(cdp_id, deposits);
        }

        pub fn remove_collateral(
            &mut self,
            cdp_proof: Proof,
            withdraw_details: Vec<(ResourceAddress, Decimal, bool)>,
        ) -> Vec<Bucket> {
            self.check_operating_status(OperatingService::RemoveCollateral);

            let cdp_id = self._validate_cdp_proof(cdp_proof);

            let (mut cdp_data, _) = self._get_cdp_data(&cdp_id, false);

            let withdrawals = withdraw_details.into_iter().fold(
                Vec::new(),
                |mut withdrawals, (pool_res_address, unit_amount, keep_deposit_unit)| {
                    let mut pool_state = self._get_pool_state(
                        &pool_res_address,
                        Some(OperatingService::RemoveCollateral),
                    );

                    let current_deposit_units = cdp_data.get_collateral_units(pool_res_address);

                    let withdraw_collateral_units = current_deposit_units.min(unit_amount);

                    cdp_data
                        .update_collateral(pool_res_address, -withdraw_collateral_units)
                        .expect("Error updating collateral for CDP");

                    let deposit_units = pool_state
                        .remove_pool_units_from_collateral(withdraw_collateral_units)
                        .expect("Error redeeming pool units from collateral");

                    let returned_assets = if !keep_deposit_unit {
                        pool_state.redeem_proxy(deposit_units)
                    } else {
                        deposit_units
                    };

                    withdrawals.push(returned_assets);

                    withdrawals
                },
            );

            let delegator_cdp_data = match cdp_data.get_type() {
                CDPType::Delegatee(delegator_data) => Some(WrappedCDPData::new(
                    &self.cdp_res_manager,
                    &delegator_data.cdp_id,
                )),
                _ => None,
            };

            CDPHealthChecker::new(
                &cdp_data,
                delegator_cdp_data.as_ref(),
                &mut self.pool_states,
            )
            .check_cdp()
            .expect("Error checking CDP");

            cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");

            withdrawals
        }

        pub fn borrow(
            &mut self,
            cdp_proof: Proof,
            borrows: Vec<(ResourceAddress, Decimal)>,
        ) -> Vec<Bucket> {
            self.check_operating_status(OperatingService::Borrow);

            let cdp_id = self._validate_cdp_proof(cdp_proof);

            let (mut cdp_data, mut delegator_cdp_data) = self._get_cdp_data(&cdp_id, true);

            let loans =
                borrows
                    .into_iter()
                    .fold(Vec::new(), |mut loans, (pool_res_address, amount)| {
                        let mut pool_state =
                            self._get_pool_state(&pool_res_address, Some(OperatingService::Borrow));

                        let (borrowed_assets, delta_loan_units) = pool_state
                            .withdraw_for_borrow(amount)
                            .expect("Error in withdraw_for_borrow");

                        cdp_data
                            .update_loan(pool_res_address, delta_loan_units)
                            .expect("Error updating loan");

                        if cdp_data.is_delegatee() {
                            delegator_cdp_data
                                .as_mut()
                                .unwrap()
                                .update_delegatee_loan(pool_res_address, delta_loan_units)
                                .expect("Error updating delegatee loan");
                        }

                        loans.push(borrowed_assets);

                        loans
                    });

            CDPHealthChecker::new(
                &cdp_data,
                delegator_cdp_data.as_ref(),
                &mut self.pool_states,
            )
            .check_cdp()
            .expect("Error checking CDP");

            cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");

            if delegator_cdp_data.is_some() {
                delegator_cdp_data
                    .as_mut()
                    .unwrap()
                    .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                    .expect("Error saving CDP");
            }

            loans
        }

        pub fn repay(
            &mut self,
            cdp_proof: Proof,
            delegatee_cdp_id: Option<NonFungibleLocalId>,
            payments: Vec<Bucket>,
        ) -> (Vec<Bucket>, Decimal) {
            self.check_operating_status(OperatingService::Repay);

            // Loan of delegatee CDP can be directly repaid by the delegator CDP
            // If the delegatee CDP is provided, we check if the delegator CDP is linked to the delegatee CDP
            let cdp_id = if let Some(delegatee_cdp_id) = delegatee_cdp_id {
                let delegator_cdp_id = self._validate_cdp_proof(cdp_proof);

                let delegatee_cdp_data: CollaterizedDebtPositionData = self
                    .cdp_res_manager
                    .get_non_fungible_data(&delegatee_cdp_id);

                match delegatee_cdp_data.cdp_type {
                    CDPType::Delegatee(delegator_info) => assert!(
                        delegator_info.cdp_id == delegator_cdp_id,
                        "Delegatee CDP not linked to provided delegator CDP"
                    ),
                    _ => panic!("Invalid delegatee CDP"),
                };

                delegatee_cdp_id
            } else {
                self._validate_cdp_proof(cdp_proof)
            };

            let (mut cdp_data, mut delegator_cdp_data) = self._get_cdp_data(&cdp_id, true);

            let (remainders, payment_value) = self._repay_internal(
                &mut cdp_data,
                &mut delegator_cdp_data,
                payments,
                None,
                false,
            );

            (remainders, payment_value)
        }

        pub fn refinance(
            &mut self,
            cdp_id: NonFungibleLocalId,
            payments: Vec<Bucket>,
        ) -> (Vec<Bucket>, Decimal) {
            let (mut cdp_data, mut delegator_cdp_data) = self._get_cdp_data(&cdp_id, true);

            CDPHealthChecker::new(
                &cdp_data,
                delegator_cdp_data.as_ref(),
                &mut self.pool_states,
            )
            .can_refinance()
            .expect("Error checking CDP");

            let (remainders, payment_value) = self._repay_internal(
                &mut cdp_data,
                &mut delegator_cdp_data,
                payments,
                None,
                false,
            );

            (remainders, payment_value)
        }

        pub fn start_liquidation(
            &mut self,
            cdp_id: NonFungibleLocalId,
            requested_collaterals: Vec<ResourceAddress>,
            total_payment_value: Option<Decimal>,
        ) -> (Vec<Bucket>, Bucket) {
            self.check_operating_status(OperatingService::Liquidation);

            if let Some(total_payment_value) = total_payment_value {
                assert!(
                    total_payment_value >= 0.into(),
                    "INVALID_INPUT: Total payment value must be non-negative"
                );
            }

            let (mut cdp_data, mut delegator_cdp_data) = self._get_cdp_data(&cdp_id, true);

            let mut cdp_health_checker = CDPHealthChecker::new(
                &cdp_data,
                delegator_cdp_data.as_ref(),
                &mut self.pool_states,
            );

            cdp_health_checker
                .can_liquidate()
                .expect("Error checking CDP");

            let temp_total_payment_value = total_payment_value
                .unwrap_or(cdp_health_checker.self_closable_loan_value)
                .min(cdp_health_checker.self_closable_loan_value);

            let (returned_collaterals, total_payement_value) = self._remove_collateral(
                delegator_cdp_data.as_mut().unwrap_or(&mut cdp_data),
                requested_collaterals,
                temp_total_payment_value,
                false,
            );

            let liquidation_term =
                self.liquidation_term_res_manager
                    .mint_ruid_non_fungible(LiquidationTerm {
                        cdp_id,
                        payement_value: total_payement_value,
                    });

            (returned_collaterals, liquidation_term)
        }

        pub fn end_liquidation(
            &mut self,
            payments: Vec<Bucket>,
            liquidation_term: Bucket,
        ) -> (Vec<Bucket>, Decimal) {
            let liquidation_term_data: LiquidationTerm =
                liquidation_term.as_non_fungible().non_fungible().data();

            let cdp_id = liquidation_term_data.cdp_id;

            let (mut cdp_data, mut delegator_cdp_data) = self._get_cdp_data(&cdp_id, true);

            let (remainders, total_payment_value) = self._repay_internal(
                &mut cdp_data,
                &mut delegator_cdp_data,
                payments,
                Some(liquidation_term_data.payement_value),
                true,
            );

            assert!(
                total_payment_value == liquidation_term_data.payement_value,
                "Total payment value does not match with the liquidation term"
            );

            if total_payment_value == liquidation_term_data.payement_value {
                self.liquidation_term_res_manager.burn(liquidation_term);

                Runtime::emit_event(CDPLiquidatedEvent {
                    cdp_id,
                    delegator_cdp_id: delegator_cdp_data
                        .as_ref()
                        .map(|delegator_cdp_data| delegator_cdp_data.clone().cdp_id),
                });
            } else {
                self.liquidation_term_res_manager.update_non_fungible_data(
                    &liquidation_term.as_non_fungible().non_fungible_local_id(),
                    "payement_value",
                    LiquidationTerm {
                        cdp_id,
                        payement_value: liquidation_term_data.payement_value - total_payment_value,
                    },
                );
            }

            (remainders, total_payment_value)
        }

        pub fn fast_liquidation(
            &mut self,
            cdp_id: NonFungibleLocalId,
            payments: Vec<Bucket>,
            requested_collaterals: Vec<ResourceAddress>,
        ) -> (Vec<Bucket>, Decimal) {
            self.check_operating_status(OperatingService::Liquidation);

            let (mut cdp_data, mut delegator_cdp_data) = self._get_cdp_data(&cdp_id, true);

            CDPHealthChecker::new(
                &cdp_data,
                delegator_cdp_data.as_ref(),
                &mut self.pool_states,
            )
            .can_liquidate()
            .expect("Error checking CDP");

            let (remainders, total_payment_value) =
                self._repay_internal(&mut cdp_data, &mut delegator_cdp_data, payments, None, true);

            let (_returned_collaterals, _total_payement_value) = self._remove_collateral(
                delegator_cdp_data.as_mut().unwrap_or(&mut cdp_data),
                requested_collaterals,
                total_payment_value,
                true,
            );

            (remainders, total_payment_value)
        }

        //*  PRIVATE UTILITY METHODS   *//

        fn _deposit_internal(&mut self, cdp_id: NonFungibleLocalId, deposits: Vec<Bucket>) {
            self.check_operating_status(OperatingService::AddCollateral);

            let (mut cdp_data, _) = self._get_cdp_data(&cdp_id, false);

            // AddCollateral to delegatee CDP is not allowed for consistency reason.
            // Delagator and delegatee CDPs should have consistent health status
            assert!(
                !cdp_data.is_delegatee(),
                "Delegatee CDP can not add collateral"
            );

            deposits.into_iter().fold((), |_, assets| {
                let res_address = assets.resource_address();

                let value = self.pool_unit_refs.get(&res_address);

                let (pool_res_address, pool_unit_res_address) = if let Some(value) = value {
                    (res_address, *value)
                } else {
                    (
                        *self.revers_pool_unit_refs.get(&res_address).unwrap(),
                        res_address,
                    )
                };

                let mut pool_state =
                    self._get_pool_state(&pool_res_address, Some(OperatingService::AddCollateral));

                let deposit_units = if res_address == pool_unit_res_address {
                    assets
                } else {
                    pool_state
                        .contribute_proxy(assets)
                        .expect("Error contributing to pool")
                };

                cdp_data
                    .update_collateral(pool_res_address, deposit_units.amount())
                    .expect("Error updating collateral for CDP");

                pool_state
                    .add_pool_units_as_collateral(deposit_units)
                    .expect("Error adding pool units as collateral");
            });

            cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");
        }

        fn _remove_collateral(
            &mut self,
            cdp_data: &mut WrappedCDPData,
            requested_collaterals: Vec<ResourceAddress>,
            requested_collaterals_value: Decimal,
            check_requested_collaterals: bool,
        ) -> (Vec<Bucket>, Decimal) {
            let mut returned_collaterals: Vec<Bucket> = Vec::new();
            let mut returned_collaterals_value = dec!(0);

            let mut temp_requested_value = requested_collaterals_value;

            for pool_res_address in requested_collaterals {
                // Make sure that that each requested collateral will have a bucket in the worktop
                if temp_requested_value == dec!(0) {
                    returned_collaterals.push(Bucket::new(pool_res_address));
                    break;
                }

                let mut pool_state =
                    self._get_pool_state(&pool_res_address, Some(OperatingService::Liquidation));

                let bonus_rate = dec!(1) + pool_state.pool_config.liquidation_bonus_rate;

                let unit_ratio = pool_state.pool.get_pool_unit_ratio();

                let max_collateral_units = cdp_data.get_collateral_units(pool_res_address);

                let max_collateral_amount = (max_collateral_units / unit_ratio)
                    .checked_truncate(RoundingMode::ToNearestMidpointToEven)
                    .unwrap();

                let mut max_collateral_value = max_collateral_amount * pool_state.last_price;

                max_collateral_value = max_collateral_value.min(bonus_rate * temp_requested_value);

                temp_requested_value -= max_collateral_value / bonus_rate;

                returned_collaterals_value += max_collateral_value / bonus_rate;

                let collateral_units = ((max_collateral_value / pool_state.last_price)
                    * unit_ratio)
                    .checked_truncate(RoundingMode::ToNearestMidpointToEven)
                    .unwrap();

                // info!("{},{}", temp_total_payment_value, total_payement_value);

                cdp_data
                    .update_collateral(pool_res_address, -collateral_units)
                    .expect("Error updating collateral for CDP");

                let pool_unit = pool_state
                    .remove_pool_units_from_collateral(collateral_units)
                    .expect("Error redeeming pool units from collateral");

                let mut collaterals = pool_state.redeem_proxy(pool_unit);
                let protocol_fee_amount =
                    collaterals.amount() * pool_state.pool_config.protocol_liquidation_fee_rate;

                pool_state.reserve.put(collaterals.take_advanced(
                    protocol_fee_amount,
                    WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
                ));

                returned_collaterals.push(collaterals);
            }

            if check_requested_collaterals {
                assert!(
                    temp_requested_value == dec!(0),
                    "Insufficient collateral value, {} remaining",
                    temp_requested_value
                );
            }

            cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");

            (returned_collaterals, returned_collaterals_value)
        }

        fn _repay_internal(
            &mut self,
            cdp_data: &mut WrappedCDPData,
            delegator_cdp_data: &mut Option<WrappedCDPData>,
            payments: Vec<Bucket>,
            payment_value: Option<Decimal>,
            for_liquidation: bool,
        ) -> (Vec<Bucket>, Decimal) {
            let mut expected_payment_value = payment_value.unwrap_or(dec!(0));

            let (remainders, total_payment_value) = payments.into_iter().fold(
                (Vec::new(), Decimal::zero()),
                |(mut remainders, mut total_payment_value), mut payment| {
                    let pool_res_address = payment.resource_address();

                    let mut pool_state = self._get_pool_state(&pool_res_address, None);

                    // ! Liquidation
                    if for_liquidation {
                        pool_state
                            .check_operating_status(OperatingService::Liquidation)
                            .expect("Liquidation is not enabled for the pool");

                    // ! Repay
                    } else {
                        pool_state
                            .check_operating_status(OperatingService::Repay)
                            .expect("Borrow is not enabled for the pool");
                    }

                    let unit_ratio = pool_state
                        .get_loan_unit_ratio()
                        .expect("Error getting loan unit ratio for provided resource");

                    let (_, pool_borrowed_amount) = pool_state.pool.get_pooled_amount();

                    let position_loan_units = cdp_data.get_loan_unit(pool_res_address);

                    let mut max_loan_amount = (position_loan_units / unit_ratio)
                        .checked_truncate(RoundingMode::ToNearestMidpointToEven)
                        .unwrap();

                    // ! Liquidation
                    if for_liquidation {
                        max_loan_amount *= pool_state.pool_config.loan_close_factor;
                    }

                    max_loan_amount = max_loan_amount.min(payment.amount());

                    let mut max_loan_value = (max_loan_amount * pool_state.last_price)
                        .min(pool_borrowed_amount * pool_state.last_price);

                    // ! Liquidation
                    if payment_value.is_some() {
                        max_loan_value = max_loan_value.min(expected_payment_value);
                        expected_payment_value -= max_loan_value;

                        assert!(
                            expected_payment_value >= dec!(0),
                            "expected_payment_value should not be negative"
                        );
                    };

                    max_loan_amount = max_loan_value / pool_state.last_price;

                    let delta_loan_unit = pool_state
                        .deposit_for_repay(payment.take_advanced(
                            max_loan_amount,
                            WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
                        ))
                        .expect("Error in deposit_from_repay");

                    cdp_data
                        .update_loan(pool_res_address, -delta_loan_unit)
                        .expect("Error updating loan");

                    if cdp_data.is_delegatee() {
                        delegator_cdp_data
                            .as_mut()
                            .unwrap()
                            .update_delegatee_loan(pool_res_address, -delta_loan_unit)
                            .expect("Error updating delegatee loan");
                    }

                    remainders.push(payment);

                    total_payment_value += max_loan_value;

                    info!(
                        "total_payment_value: {}, expected_payment_value: {}",
                        total_payment_value,
                        expected_payment_value // delta_loan_unit,
                                               // cdp_data.get_loan_unit(pool_res_address)
                    );

                    (remainders, total_payment_value)
                },
            );

            if payment_value.is_some() {
                assert!(
                    expected_payment_value == dec!(0),
                    "Insufficient payment value, {} remaining",
                    expected_payment_value
                );
            }

            cdp_data
                .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                .expect("Error saving CDP");

            if cdp_data.is_delegatee() {
                delegator_cdp_data
                    .as_mut()
                    .unwrap()
                    .save_cdp(&self.cdp_res_manager, self.market_config.max_cdp_position)
                    .expect("Error saving CDP");
            }

            (remainders, total_payment_value)
        }

        fn _get_pool_state(
            &mut self,
            pool_res_address: &ResourceAddress,
            operating_status: Option<OperatingService>,
        ) -> KeyValueEntryRefMut<'_, LendingPoolState> {
            let mut pool_state = self.pool_states.get_mut(pool_res_address).unwrap();

            if let Some(operating_status) = operating_status {
                pool_state
                    .check_operating_status(operating_status)
                    .expect("Invalid operating status");
            }

            pool_state
                .update_interest_and_price()
                .expect("Error updating pool state");

            pool_state
        }

        fn _get_cdp_data(
            &mut self,
            cdp_id: &NonFungibleLocalId,
            get_delegator_cdp_data: bool,
        ) -> (WrappedCDPData, Option<WrappedCDPData>) {
            let cdp_data = WrappedCDPData::new(&self.cdp_res_manager, cdp_id);

            let delegator_cdp_data = if get_delegator_cdp_data && cdp_data.is_delegatee() {
                Some(WrappedCDPData::new(
                    &self.cdp_res_manager,
                    &cdp_data
                        .get_delegator_id()
                        .expect("Error getting delegator_id"),
                ))
            } else {
                None
            };

            (cdp_data, delegator_cdp_data)
        }

        fn _get_new_cdp_id(&mut self) -> u64 {
            self.cdp_counter += 1;
            self.cdp_counter
        }

        fn _validate_cdp_proof(&self, cdp: Proof) -> NonFungibleLocalId {
            let validated_cdp = cdp.check(self.cdp_res_manager.address());
            validated_cdp.as_non_fungible().non_fungible_local_id()
        }

        fn check_operating_status(&self, value: OperatingService) {
            assert!(
                self.operating_status.check(value.clone()),
                "{:?} is not allowed for this pool",
                value
            );
        }
    }
}
