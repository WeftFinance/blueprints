use scrypto::prelude::*;


#[derive(ScryptoSbor, NonFungibleData)]
pub struct AuthBadgeData {}

#[blueprint]
mod caviar_lsu_pool_component_faucet {

    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            
            set_liquidity_token_total_supply => restrict_to: [admin];

            set_dex_valuation_xrd => restrict_to: [admin];

            get_liquidity_token_total_supply => PUBLIC;

            get_dex_valuation_xrd => PUBLIC;
        }
    }

    struct CaviarLsuPoolComponentFaucet {
        // Define what resources and data will be managed by Hello component
        liquidity_token_total_supply : Decimal, 
        dex_valuation_xrd : Decimal
    }

    impl CaviarLsuPoolComponentFaucet {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate() -> NonFungibleBucket {
            
            let (component_address_reservation, _) =
            Runtime::allocate_component_address(CaviarLsuPoolComponentFaucet::blueprint_id());

            let (admin_badge_address_reservation, admin_badge_address) =
                Runtime::allocate_non_fungible_address();

            let admin_rule = rule!(require(admin_badge_address));

            let admin_badge = ResourceBuilder::new_integer_non_fungible::<AuthBadgeData>(
                OwnerRole::Fixed(admin_rule.clone()),
            )
            .with_address(admin_badge_address_reservation)
            .mint_initial_supply([(IntegerNonFungibleLocalId::from(1), AuthBadgeData {})]);

            
            Self {
                liquidity_token_total_supply : Decimal::from(10_000_000),
                dex_valuation_xrd : Decimal::from(13_000_000)
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
    
      pub fn set_liquidity_token_total_supply(&mut self, 
                                              liquidity_token_total_supply : Decimal){
            self.liquidity_token_total_supply = liquidity_token_total_supply; 
      }

      
      pub fn set_dex_valuation_xrd(&mut self, 
                                     dex_valuation_xrd : Decimal){
            self.dex_valuation_xrd = dex_valuation_xrd; 
        }

      pub fn get_liquidity_token_total_supply(&self) -> Decimal {
        self.liquidity_token_total_supply
      }

      pub fn get_dex_valuation_xrd(&self) -> Decimal {
         self.dex_valuation_xrd
      }
      
    }
}
