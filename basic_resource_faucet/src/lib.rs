use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AuthBadgeData {}

#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
    pub timestamp: i64,
    pub price: Decimal,
}

#[blueprint]
mod faucet {

    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            create_resource => restrict_to: [admin];
            swap => PUBLIC;
            get_resource => PUBLIC;
        }
    }

    struct Faucet {
        admin_rule: AccessRule,
        component_address: ComponentAddress,
        collected_xrd: Vault,
        price_feed: Global<AnyComponent>,
        res_address_list: Vec<ResourceManager>,
    }
    impl Faucet {
        pub fn instantiate(price_feed: Global<AnyComponent>) -> NonFungibleBucket {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Faucet::blueprint_id());

            let (admin_badge_address_reservation, admin_badge_address) =
                Runtime::allocate_non_fungible_address();

            let admin_rule = rule!(require(admin_badge_address));

            let admin_badge = ResourceBuilder::new_integer_non_fungible::<AuthBadgeData>(
                OwnerRole::Fixed(admin_rule.clone()),
            )
            .with_address(admin_badge_address_reservation)
            .mint_initial_supply([(IntegerNonFungibleLocalId::from(1), AuthBadgeData {})]);

            Self {
                admin_rule: admin_rule.clone(),
                price_feed,
                res_address_list: Vec::new(),
                component_address,
                collected_xrd: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(admin_rule))
            .with_address(address_reservation)
            .roles(roles! {
                admin => rule!(require(admin_badge.resource_address()));
            })
            .globalize();

            admin_badge
        }

        /// FAUCET : CREATE AND SUPPLY RESOURCES FOR TEST

        pub fn create_resource(
            &mut self,
            symbol: String,
            name: String,
            icon: String,
            initial_supply: Decimal,
        ) -> Bucket {
            let res_manager =
                ResourceBuilder::new_fungible(OwnerRole::Fixed(self.admin_rule.clone()))
                    .metadata(metadata!(init{
                        "symbol"=> symbol,updatable;
                        "name"=> name,updatable;
                        "icon_url"=> icon,updatable;
                    }))
                    .mint_roles(mint_roles! {
                        minter =>    rule!(require(global_caller(self.component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner =>  rule!(require(global_caller(self.component_address)));
                        burner_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply();

            self.res_address_list.push(res_manager);

            res_manager.mint(initial_supply)
        }

        fn get_price(&self, resource: ResourceAddress) -> PriceInfo {
            self.price_feed
                .call_raw::<Option<PriceInfo>>("get_price", scrypto_args!(resource))
                .expect("Price not found")
        }

        pub fn get_resource(&mut self, resource: ResourceAddress, xrd: Bucket) -> Bucket {
            assert!(
                xrd.resource_address() == XRD,
                "Provide XRD to get faucet tokens"
            );

            let from_price = self.get_price(XRD);

            let to_price = self.get_price(resource);

            let to_amount = xrd.amount() * (from_price.price / to_price.price);

            self.collected_xrd.put(xrd);

            let resource_manager = self
                .res_address_list
                .iter()
                .find(|rm| rm.address() == resource);

            resource_manager
                .expect("Resource not found")
                .mint(to_amount)
        }

        /// TEST EXCHANGE

        pub fn swap(&mut self, from_bucket: Bucket, to_resource: ResourceAddress) -> Bucket {
            let from_resource = from_bucket.resource_address();

            let from_price = self.get_price(from_resource);
            let to_price = self.get_price(to_resource);

            let to_amount = from_bucket.amount() * (from_price.price / to_price.price);

            if from_bucket.resource_address() == XRD {
                self.collected_xrd.put(from_bucket);
            } else {
                from_bucket.burn();
            }

            let resource_manager = self
                .res_address_list
                .iter()
                .find(|rm| rm.address() == to_resource);

            resource_manager
                .expect("Resource not found")
                .mint(to_amount)
        }
    }
}
