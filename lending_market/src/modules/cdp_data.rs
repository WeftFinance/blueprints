use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub enum CDPUpdatedEvenType {
    AddCollateral,
    RemoveCollateral,
    Borrow,
    Repay,
    Liquidate,
    Refinance,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct CDPUpdatedEvent {
    pub cdp_id: NonFungibleLocalId,
    pub event_type: CDPUpdatedEvenType,
}

#[derive(ScryptoSbor, Clone, PartialEq, Debug)]
pub struct DelegatorInfo {
    pub cdp_id: NonFungibleLocalId,
    pub delegatee_index: u64,
    pub max_loan_value: Option<Decimal>,
    pub max_loan_value_ratio: Option<Decimal>,
}

#[derive(ScryptoSbor, Clone, PartialEq, Debug)]
pub struct DelegateeInfo {
    pub delegatee_count: u64,
    pub linked_count: u64,
}

#[derive(ScryptoSbor, Clone, PartialEq, Debug)]
pub enum CDPType {
    Standard,
    Delegator(DelegateeInfo),
    Delegatee(DelegatorInfo),
}
impl CDPType {
    pub fn is_delegator(&self) -> bool {
        matches!(self, CDPType::Delegator(_))
    }

    pub fn is_delegatee(&self) -> bool {
        matches!(self, CDPType::Delegatee(_))
    }
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct CollaterizedDebtPositionData {
    #[mutable]
    pub key_image_url: String,

    #[mutable]
    pub name: String,

    #[mutable]
    pub description: String,

    // #[immutable]
    pub minted_at: i64,

    #[mutable]
    pub updated_at: i64,

    #[mutable]
    pub cdp_type: CDPType,

    #[mutable]
    pub collaterals: IndexMap<ResourceAddress, Decimal>,

    #[mutable]
    pub loans: IndexMap<ResourceAddress, Decimal>,

    #[mutable]
    pub delegatee_loans: IndexMap<ResourceAddress, Decimal>,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct WrappedCDPData {
    pub cdp_data: CollaterizedDebtPositionData,
    pub cdp_id: NonFungibleLocalId,
    pub cdp_type_updated: bool,
    pub collateral_updated: bool,
    pub loan_updated: bool,
    pub delegatee_loan_updated: bool,
}

impl WrappedCDPData {
    pub fn new(res_manager: &ResourceManager, cdp_id: &NonFungibleLocalId) -> WrappedCDPData {
        let cdp_data = res_manager.get_non_fungible_data(cdp_id);
        WrappedCDPData {
            cdp_id: cdp_id.clone(),
            cdp_data,
            cdp_type_updated: false,
            collateral_updated: false,
            loan_updated: false,
            delegatee_loan_updated: false,
        }
    }

    pub fn is_delegatee(&self) -> bool {
        self.cdp_data.cdp_type.is_delegatee()
    }

    pub fn get_type(&self) -> CDPType {
        self.cdp_data.cdp_type.clone()
    }

    pub fn get_data(&self) -> CollaterizedDebtPositionData {
        self.cdp_data.clone()
    }

    pub fn get_delegator_id(&self) -> Result<NonFungibleLocalId, String> {
        match &self.cdp_data.cdp_type {
            CDPType::Delegatee(delegator_info) => Ok(delegator_info.cdp_id.clone()),
            _ => Err("WrappedCDPData/get_delegator_id: CDP is not delegatee".into()),
        }
    }

    fn get_units(map: &IndexMap<ResourceAddress, Decimal>, key: ResourceAddress) -> Decimal {
        map.get(&key).copied().unwrap_or(Decimal::ZERO)
    }

    pub fn get_collateral_units(&self, collateral: ResourceAddress) -> Decimal {
        Self::get_units(&self.cdp_data.collaterals, collateral)
    }

    pub fn get_loan_unit(&self, loan: ResourceAddress) -> Decimal {
        Self::get_units(&self.cdp_data.loans, loan)
    }

    //

    pub fn increase_delegatee_count(&mut self) -> Result<(u64, u64), String> {
        let result = match &mut self.cdp_data.cdp_type {
            CDPType::Delegator(delegatee_info) => {
                delegatee_info.delegatee_count += 1;
                delegatee_info.linked_count += 1;
                Ok((delegatee_info.delegatee_count, delegatee_info.linked_count))
            }
            CDPType::Standard => {
                self.cdp_data.cdp_type = CDPType::Delegator(DelegateeInfo {
                    delegatee_count: 1,
                    linked_count: 1,
                });
                Ok((1u64, 1u64))
            }
            CDPType::Delegatee(_) => {
                Err("WrappedCDPData/increase_delegatee_count: CDP is not delegator".into())
            }
        };

        self.cdp_type_updated = true;

        result
    }

    pub fn decrease_delegatee_count(&mut self) -> Result<(), String> {
        let result = match &mut self.cdp_data.cdp_type {
            CDPType::Delegator(delegatee_info) => {
                if delegatee_info.delegatee_count == 1 {
                    self.cdp_data.cdp_type = CDPType::Standard;
                } else {
                    delegatee_info.delegatee_count -= 1;
                }
                Ok(())
            }

            _ => Err("WrappedCDPData/increase_delegatee_count: CDP is not delegator".into()),
        };

        self.cdp_type_updated = true;

        result
    }

    pub fn update_cdp_type(&mut self, cdp_type: CDPType) {
        self.cdp_data.cdp_type = cdp_type;
        self.cdp_type_updated = true;
    }

    pub fn update_delegatee_info(
        &mut self,
        max_loan_value: Option<Decimal>,
        max_loan_value_ratio: Option<Decimal>,
    ) -> Result<(), String> {
        if let CDPType::Delegatee(delegatee_info) = &mut self.cdp_data.cdp_type {
            delegatee_info.max_loan_value = max_loan_value;
            delegatee_info.max_loan_value_ratio = max_loan_value_ratio;

            self.cdp_type_updated = true;
            Ok(())
        } else {
            Err("WrappedCDPData/update_delegatee_info: CDP is not delegatee".into())
        }
    }

    pub fn update_collateral(
        &mut self,
        res_address: ResourceAddress,
        units: Decimal,
    ) -> Result<(), String> {
        let result = Self::update_map(&mut self.cdp_data.collaterals, res_address, units);
        self.collateral_updated = true;
        result
    }

    pub fn update_loan(
        &mut self,
        res_address: ResourceAddress,
        units: Decimal,
    ) -> Result<(), String> {
        let result = Self::update_map(&mut self.cdp_data.loans, res_address, units);
        self.loan_updated = true;
        result
    }

    pub fn update_delegatee_loan(
        &mut self,
        res_address: ResourceAddress,
        units: Decimal,
    ) -> Result<(), String> {
        let result = Self::update_map(&mut self.cdp_data.delegatee_loans, res_address, units);
        self.delegatee_loan_updated = true;
        result
    }

    pub fn save_cdp(
        &self,
        res_manager: &ResourceManager,
        max_cdp_position: u8,
    ) -> Result<(), String> {
        let mut updated = false;

        if self.cdp_type_updated {
            res_manager.update_non_fungible_data(
                &self.cdp_id,
                "cdp_type",
                self.cdp_data.cdp_type.clone(),
            );
            updated = true;
        }

        if self.collateral_updated {
            res_manager.update_non_fungible_data(
                &self.cdp_id,
                "collaterals",
                self.cdp_data.collaterals.clone(),
            );
            updated = true;
        }

        if self.loan_updated {
            res_manager.update_non_fungible_data(
                &self.cdp_id,
                "loans",
                self.cdp_data.loans.clone(),
            );
            updated = true;
        }

        if self.delegatee_loan_updated {
            res_manager.update_non_fungible_data(
                &self.cdp_id,
                "delegatee_loans",
                self.cdp_data.delegatee_loans.clone(),
            );
            updated = true;
        }

        if updated {
            let position_count = self.cdp_data.collaterals.len()
                + self.cdp_data.loans.len()
                + self.cdp_data.delegatee_loans.len();

            assert!(position_count as u8 <= max_cdp_position);

            res_manager.update_non_fungible_data(
                &self.cdp_id,
                "updated_at",
                Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch,
            );
        }

        Ok(())
    }

    // local methods

    fn update_map(
        map: &mut IndexMap<ResourceAddress, Decimal>,
        key: ResourceAddress,
        units: Decimal,
    ) -> Result<(), String> {
        if units == Decimal::ZERO {
            return Ok(());
        }

        if let Some(entry) = map.get_mut(&key) {
            *entry += units;

            if *entry < Decimal::ZERO {
                return Err(
                    "WrappedCDPData/update_map: entry must be greater than or equal to 0".into(),
                );
            }

            if *entry == Decimal::ZERO {
                map.remove(&key);
            }
        } else {
            map.insert(key, units);
        }

        Ok(())
    }
}
