use crate::modules::utils::is_valid_rate;
use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub enum UpdateLiquidationThresholdInput {
    DefaultValue(Decimal),
    IdenticalResource(Option<Decimal>),
    IdenticalAssetType(Option<Decimal>),
    ResourceEntry(ResourceAddress, Option<Decimal>),
    AssetTypeEntry(u8, Option<Decimal>),
}

#[derive(ScryptoSbor, Clone)]
pub struct LiquidationThreshold {
    pub identical_resource: Option<Decimal>,
    pub identical_asset_type: Option<Decimal>,
    pub resource: IndexMap<ResourceAddress, Decimal>,
    pub asset_type: IndexMap<u8, Decimal>,
    pub default_value: Decimal,
}
impl LiquidationThreshold {
    pub fn check(&self) -> Result<(), String> {
        if !is_valid_rate(self.default_value) {
            return Err("Invalid liquidation threshold default value".into());
        }

        if self.identical_resource.is_some() && !is_valid_rate(self.identical_resource.unwrap()) {
            return Err("Invalid liquidation threshold identical resource".into());
        };

        if self.identical_asset_type.is_some() && !is_valid_rate(self.identical_asset_type.unwrap())
        {
            return Err("Invalid liquidation threshold identical asset type".into());
        };

        for (_, threshold) in self.resource.iter() {
            if !is_valid_rate(*threshold) {
                return Err("Invalid liquidation threshold resource entry".into());
            }
        }

        for (_, threshold) in self.asset_type.iter() {
            if !is_valid_rate(*threshold) {
                return Err("Invalid liquidation threshold asset type entry".into());
            }
        }

        Ok(())
    }

    pub fn get_ratio(
        &self,
        collateral_res_address: ResourceAddress,
        collateral_asset_type: u8,
        loan_res_address: ResourceAddress,
        loan_asset_type: u8,
    ) -> Decimal {
        if loan_res_address == collateral_res_address {
            if self.identical_resource.is_some() {
                return self.identical_resource.unwrap();
            } else {
                return 0.into();
            }
        }

        if self.identical_asset_type.is_some() && loan_asset_type == collateral_asset_type {
            return self.identical_asset_type.unwrap();
        }

        self.resource
            .get(&loan_res_address)
            .copied()
            .unwrap_or_else(|| {
                self.asset_type
                    .get(&loan_asset_type)
                    .copied()
                    .unwrap_or(self.default_value)
            })
    }

    pub fn update_liquidation_threshold(
        &mut self,
        value: UpdateLiquidationThresholdInput,
    ) -> Result<(), String> {
        match value {
            UpdateLiquidationThresholdInput::DefaultValue(value) => {
                self.default_value = value;
            }

            UpdateLiquidationThresholdInput::IdenticalResource(value) => {
                self.identical_resource = value;
            }

            UpdateLiquidationThresholdInput::IdenticalAssetType(value) => {
                self.identical_asset_type = value;
            }

            UpdateLiquidationThresholdInput::AssetTypeEntry(asset_type, value) => {
                self.set_asset_type_entry(asset_type, value);
            }

            UpdateLiquidationThresholdInput::ResourceEntry(res_address, value) => {
                self.set_resource_entry(res_address, value);
            }
        }

        self.check()?;

        Ok(())
    }

    fn set_resource_entry(&mut self, resource: ResourceAddress, threshold: Option<Decimal>) {
        if let Some(threshold) = threshold {
            self.resource.insert(resource, threshold);
        } else {
            self.resource.remove(&resource);
        }
    }

    fn set_asset_type_entry(&mut self, asset_type: u8, threshold: Option<Decimal>) {
        if let Some(threshold) = threshold {
            self.asset_type.insert(asset_type, threshold);
        } else {
            self.asset_type.remove(&asset_type);
        }
    }
}
