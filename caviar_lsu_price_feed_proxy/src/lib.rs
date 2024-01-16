use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
    pub timestamp: i64,
    pub price: Decimal,
}

#[blueprint]
mod caviar_lsu_price_feed_proxy {

    enable_method_auth! {
        methods {
            get_price => PUBLIC;
        }
    }

    struct CaviarLsuPriceFeedProxy {
        prices: IndexMap<ResourceAddress, PriceInfo>,
        lsu_resource_address: ResourceAddress,
        lsu_pool_component_address: ComponentAddress,
    }

    impl CaviarLsuPriceFeedProxy {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate(
            lsu_resource_address: ResourceAddress,
            lsu_pool_component_address: ComponentAddress,
        ) -> Global<CaviarLsuPriceFeedProxy> {
            let mut component_by_resource_address: IndexMap<ResourceAddress, ComponentAddress> =
                IndexMap::new();
            component_by_resource_address.insert(lsu_resource_address, lsu_pool_component_address);

            Self {
                prices: IndexMap::new(),
                lsu_resource_address,
                lsu_pool_component_address,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn get_price(&mut self, resource_address: ResourceAddress) -> Option<PriceInfo> {
            assert!(
                self.lsu_resource_address == resource_address,
                "Invalid resource address"
            );

            let caviar_pool_component: Global<AnyComponent> =
                self.lsu_pool_component_address.into();
            let valuation_in_xrd =
                caviar_pool_component.call_raw::<Decimal>("get_dex_valuation_xrd", scrypto_args!());
            let total_supply: Decimal = caviar_pool_component
                .call_raw::<Decimal>("get_liquidity_token_total_supply", scrypto_args!());
            if total_supply <= Decimal::zero() {
                return None;
            }

            let now = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;
            let price_info = PriceInfo {
                price: valuation_in_xrd / total_supply,
                timestamp: now,
            };

            self.prices.insert(resource_address, price_info.clone());

            return Some(price_info);
        }
    }
}
