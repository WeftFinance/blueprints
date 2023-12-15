use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AuthBadgeData {}

#[blueprint]
mod caviar_lsu_pool_component_faucet {

    enable_method_auth! {
        methods {
            get_liquidity_token_total_supply => PUBLIC;

            get_dex_valuation_xrd => PUBLIC;
        }
    }

    struct CaviarLsuPoolComponentFaucet {
        // Define what resources and data will be managed by Hello component
        liquidity_token_total_supply: Decimal,
        dex_valuation_xrd: Decimal,
    }

    impl CaviarLsuPoolComponentFaucet {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate(
            liquidity_token_total_supply: Decimal,
            dex_valuation_xrd: Decimal,
        ) -> Global<CaviarLsuPoolComponentFaucet> {
            Self {
                liquidity_token_total_supply: Decimal::from(liquidity_token_total_supply),
                dex_valuation_xrd: Decimal::from(dex_valuation_xrd),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn get_liquidity_token_total_supply(&self) -> Decimal {
            self.liquidity_token_total_supply
        }

        pub fn get_dex_valuation_xrd(&self) -> Decimal {
            self.dex_valuation_xrd
        }
    }
}
