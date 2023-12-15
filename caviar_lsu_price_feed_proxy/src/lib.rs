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
         "package_sim1ph6xspj0xlmspjju2asxg7xnucy7tk387fufs4jrfwsvt85wvqf70a",// resim batch 
        //"package_rdx1pkfrtmv980h85c9nvhxa7c9y0z4vxzt25c3gdzywz5l52g5t0hdeey", // mainnet
        // "package_sim1ph8fqgwl6sdmlxxv06sf2sgk3jp9l5vrrc2enpqm5hx686auz0d9k5", // testing
        CaviarLsuPoolComponent {
            fn get_dex_valuation_xrd(&self) -> Decimal;
            fn get_liquidity_token_total_supply(&self) -> Decimal;
        }
    );

    enable_method_auth! {
        methods {

            get_price => PUBLIC;
        }
    }

    struct CaviarLsuPriceFeedProxy {
        component_by_resource_address: IndexMap<ResourceAddress, ComponentAddress>,
    }

    impl CaviarLsuPriceFeedProxy {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate(lsu_resource_address : ResourceAddress, 
                           lsu_pool_component_address : ComponentAddress) -> Global<CaviarLsuPriceFeedProxy> {
            
            let mut component_by_resource_address:  IndexMap<ResourceAddress, ComponentAddress> = IndexMap::new(); 
            component_by_resource_address.insert(lsu_resource_address,lsu_pool_component_address);

            Self {
                component_by_resource_address,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()

        }

        pub fn get_price(&mut self, resource_address: ResourceAddress) -> Option<PriceInfo> {
            match self.component_by_resource_address.get(&resource_address) {
                None => None,
                Some(component_address) => {
                    let caviar_pool_component: Global<AnyComponent> =
                        (*component_address).into();
                    let valuation_in_xrd = caviar_pool_component.call_raw::<Decimal>("get_dex_valuation_xrd", scrypto_args!());
                    let total_supply: Decimal =
                    caviar_pool_component.call_raw::<Decimal>("get_liquidity_token_total_supply", scrypto_args!());
                    if total_supply <= Decimal::zero() {
                        return None;
                    }

                    let now = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;
                    let price_info = PriceInfo {
                        price: valuation_in_xrd / total_supply,
                        timestamp: now,
                    };
                    return Some(price_info);
                }
            }
        }

      
    }
}
