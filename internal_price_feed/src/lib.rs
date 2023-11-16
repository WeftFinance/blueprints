use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
    pub timestamp: i64,
    pub price: Decimal,
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AuthBadgeData {}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct UpdaterBadgeData {
    pub active: bool,
}

#[blueprint]
mod price_feed {

    enable_method_auth! {
        roles {
            admin => updatable_by: [];
            updater => updatable_by: [admin];
        },
        methods {
            mint_updater_badge => restrict_to: [admin];
            update_updater_badge => restrict_to: [admin];
            admin_update_price => restrict_to: [admin];

            update_price => restrict_to: [updater];

            get_price => PUBLIC;
        }
    }

    pub struct PriceFeed {
        prices: IndexMap<ResourceAddress, PriceInfo>,
        updater_badge_manager: ResourceManager,
        updater_counter: u64,
    }

    impl PriceFeed {
        pub fn instantiate() -> NonFungibleBucket {
            let (component_address_reservation, component_address) =
                Runtime::allocate_component_address(PriceFeed::blueprint_id());

            let component_rule = rule!(require(global_caller(component_address)));

            let (admin_badge_address_reservation, admin_badge_address) =
                Runtime::allocate_non_fungible_address();

            let admin_rule = rule!(require(admin_badge_address));

            let admin_badge = ResourceBuilder::new_integer_non_fungible::<AuthBadgeData>(
                OwnerRole::Fixed(admin_rule),
            )
            .with_address(admin_badge_address_reservation)
            .mint_initial_supply([(IntegerNonFungibleLocalId::from(1), AuthBadgeData {})]);

            let admin_rule = rule!(require(admin_badge.resource_address()));

            let updater_badge_manager =
                ResourceBuilder::new_integer_non_fungible::<UpdaterBadgeData>(OwnerRole::Fixed(
                    admin_rule.clone(),
                ))
                .mint_roles(mint_roles! {
                    minter => component_rule.clone();
                    minter_updater =>  rule!(deny_all);
                })
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                  non_fungible_data_updater => component_rule;
                  non_fungible_data_updater_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            Self {
                prices: IndexMap::new(),
                updater_badge_manager,
                updater_counter: 0,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(admin_rule.clone()))
            .with_address(component_address_reservation)
            .roles(roles! {
                admin => admin_rule;
                updater => rule!(require(updater_badge_manager.address()));
            })
            .globalize();

            admin_badge
        }

        // * Admin Methods * //

        pub fn mint_updater_badge(&mut self, active: bool) -> Bucket {
            let badge_id = NonFungibleLocalId::Integer(self._get_new_id().into());

            self.updater_badge_manager
                .mint_non_fungible(&badge_id, UpdaterBadgeData { active })
        }

        pub fn update_updater_badge(&self, local_id: NonFungibleLocalId, active: bool) {
            self.updater_badge_manager
                .update_non_fungible_data(&local_id, "active", active);
        }

        pub fn admin_update_price(&mut self, resource: ResourceAddress, price: Decimal) {
            let now = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;
            self.prices.insert(
                resource,
                PriceInfo {
                    timestamp: now,
                    price,
                },
            );
        }

        // * Updater Methods * //

        pub fn update_price(
            &mut self,
            badge_proof: Proof,
            resource: ResourceAddress,
            price: Decimal,
        ) {
            let local_id = badge_proof
                .check(self.updater_badge_manager.address())
                .as_non_fungible()
                .non_fungible_local_id();

            let badge_data: UpdaterBadgeData =
                self.updater_badge_manager.get_non_fungible_data(&local_id);

            assert!(badge_data.active, "Updater badge is not active.");
            let now = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;
            self.prices.insert(
                resource,
                PriceInfo {
                    timestamp: now,
                    price,
                },
            );
        }

        // * Public Methods * //

        pub fn get_price(&self, quote: ResourceAddress) -> Option<PriceInfo> {
            self.prices.get(&quote).cloned()
        }

        // * Helpers * //

        fn _get_new_id(&mut self) -> u64 {
            self.updater_counter += 1;
            self.updater_counter
        }
    }
}
