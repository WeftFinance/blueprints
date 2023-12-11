use scrypto::prelude::*;


#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
    pub timestamp: i64,
    pub price: Decimal,
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AuthBadgeData {}

#[blueprint]
mod caviar_lsu_price_feed_proxy {
    extern_blueprint!(

        // "package_tdx_2_1p4wnzxlrcv9s6hsy7fdv8td06up4wzwe5vjpmw8f8jgyj4z6vhqnl5",  // stokenet
        // "package_sim1ph6xspj0xlmspjju2asxg7xnucy7tk387fufs4jrfwsvt85wvqf70a",// resim batch
        "package_sim1pkwaf2l9zkmake5h924229n44wp5pgckmpn0lvtucwers56awywems", // resim sdk
        // "package_sim1ph8fqgwl6sdmlxxv06sf2sgk3jp9l5vrrc2enpqm5hx686auz0d9k5", // testing
        CaviarLsuPoolComponent {
            fn get_dex_valuation_xrd(&self) -> Decimal;
            fn get_liquidity_token_total_supply(&self) -> Decimal;
        }
    );

    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {

            add_resource_address => restrict_to: [admin];

            get_price => PUBLIC;
        }
    }

    struct CaviarLsuPriceFeedProxy {
        component_by_resource_address :  IndexMap<ResourceAddress, ComponentAddress>
    }


    impl CaviarLsuPriceFeedProxy {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate() -> NonFungibleBucket  {

            let (component_address_reservation, _) =
            Runtime::allocate_component_address(CaviarLsuPriceFeedProxy::blueprint_id());

            let (admin_badge_address_reservation, admin_badge_address) =
                Runtime::allocate_non_fungible_address();

            let admin_rule = rule!(require(admin_badge_address));

            let admin_badge = ResourceBuilder::new_integer_non_fungible::<AuthBadgeData>(
                OwnerRole::Fixed(admin_rule.clone()),
            )
            .with_address(admin_badge_address_reservation)
            .metadata(metadata!(init{
                "description"=> "Representing the weft Caviar Lsu Price Feed Proxy admin badge",updatable;
                "name"=> "Weft Caviar LSU Price Feed Proxy",updatable;
                "icon_url"=> "https://res.cloudinary.com/daisvxhyu/image/upload/v1696647342/weft/icons/icon-weft.png",updatable;
            }))
            .mint_initial_supply([(IntegerNonFungibleLocalId::from(2), AuthBadgeData {})]);

            Self {
                component_by_resource_address : IndexMap::new()
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(admin_rule.clone()))
            .with_address(component_address_reservation)
            .roles(roles! {
                admin => admin_rule;
            })
            .globalize();

            admin_badge
        }

        pub fn get_price(&mut self, resource_address: ResourceAddress) -> Option<PriceInfo> {
            
            match self.component_by_resource_address.get(&resource_address) {
                None => None , 
                Some(component_address) =>  {
                    let caviar_pool_component : Global<CaviarLsuPoolComponent> =  (*component_address).into(); 
                    let valuation_in_xrd = caviar_pool_component.get_dex_valuation_xrd();
                    let total_supply : Decimal = caviar_pool_component.get_liquidity_token_total_supply();
                    if total_supply == Decimal::zero() {
                        return None; 
                    } 

                    let now = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;
                    let price_info  =  PriceInfo {
                        price : valuation_in_xrd / total_supply ,
                        timestamp : now
                    };
                    return Some(price_info);
                }
            }

        }

        pub fn add_resource_address(&mut self, 
                                     resource_address : ResourceAddress, 
                                     component_address : ComponentAddress) {
            let already_exist = !self.component_by_resource_address.get(&resource_address).is_none(); 
            if already_exist {
                self.component_by_resource_address.remove(&resource_address); 
            }
             self.component_by_resource_address.insert(resource_address, component_address); 
        }
    }
}
