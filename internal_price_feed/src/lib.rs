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

// Recuperer les information de prix resource (xrd/usdt/usdc) en s'appuyant sure des api externes
// et les ramener dans un environment exploitable depuis des composants Radix

#[blueprint]
mod price_feed {

// structure pour definir les autorisations pour chaque method definies
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
    
    // la structure permettant d'acceder aux information de l'instance de code.
    // elle reste contextuelle a une instance precise et persistence pour permettre une exploitation
    pub struct PriceFeed {
        prices: IndexMap<ResourceAddress, PriceInfo>,
        updater_badge_manager: ResourceManager,
        updater_counter: u64,
    }

    impl PriceFeed {
        pub fn instantiate() -> NonFungibleBucket {

        // reservation d'address effectue par le bias d'une function standard.
            let (component_address_reservation /*information permettant de creer l'address*/, 
                component_address/*address effectivement sollicitee*/) /* le retour de la function*/ 
                =
                Runtime::allocate_component_address(PriceFeed::blueprint_id()) /*la function d'appel qui ramene le couple.'*/;

                // la creation de la regle associee au composant

            let component_rule /*le nom de la regle*/ = 
            
            rule!( /* utilisation de la macro rule! qui permet la generation d'un code' */
                require( /*indique que la regle est requise*/
                    global_caller( /*indique que l'appel depuis un acteur virtuel (tous les appels qui ne sont pas faite par un homme)'*/
                        component_address /*exploitation de l'address reservee pour creer la regle'*/
                    )
                )
            );

            let (admin_badge_address_reservation, admin_badge_address) =
                Runtime::allocate_non_fungible_address();
            // la creation de regle associee a une resource utilisee comme badge administrateur
            let admin_rule = rule!(
                require(
                    admin_badge_address
                )
            );


            // permettre d'attribuer le role owner a la resource en cours de creation
            let admin_badge = ResourceBuilder::new_integer_non_fungible::<AuthBadgeData>(
                OwnerRole::Fixed(admin_rule.clone()),
            )
            .with_address(admin_badge_address_reservation)
            .mint_initial_supply([(IntegerNonFungibleLocalId::from(1), AuthBadgeData {})]);

            // permet de definir des regles de gestion des badges lorsqu'on souhaite un comportement differents des attributes par default
            let updater_badge_manager =
                ResourceBuilder::new_integer_non_fungible::<UpdaterBadgeData>(OwnerRole::Fixed(
                    admin_rule.clone(), //
                )) 
                .mint_roles(mint_roles! {
                    minter => component_rule.clone();   // seul le composant en cours peut creer un badge de update du prix
                    minter_updater =>  rule!(deny_all); // personne ne peut changer la regle
                })
                .non_fungible_data_update_roles(non_fungible_data_update_roles! { 
                  non_fungible_data_updater => component_rule; 
                  non_fungible_data_updater_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            // initialisation effective de l'instance
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

        // permet de creer un badge
        pub fn mint_updater_badge(&mut self, active: bool) -> Bucket {
            let badge_id = NonFungibleLocalId::Integer(self._get_new_id().into());

            self.updater_badge_manager
                .mint_non_fungible(&badge_id, UpdaterBadgeData { active })
        }

        // permet de faire une mise a jour d'un badge existent
        pub fn update_updater_badge(&self, local_id: NonFungibleLocalId, active: bool) {
            self.updater_badge_manager
                .update_non_fungible_data(&local_id, "active", active);
        }

        // permet de definir le prix en exploitant les privileges d'administrateur
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
        // permet de mettre a jour le prix en exploitant le badge genere par l'adinistrateur plus haut
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

        // permet d'obtenir le prix d'une resource
        pub fn get_price(&self, quote: ResourceAddress) -> Option<PriceInfo> {
            let price = self.prices.get(&quote);

            price.cloned()
        }

        // * Helpers * //
        // permet d'obtenir un nouvel ID
        fn _get_new_id(&mut self) -> u64 {
            self.updater_counter += 1;
            self.updater_counter
        }
    }
}
