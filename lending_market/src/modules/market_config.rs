use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub enum UpdateMarketConfigInput {
    MaxCDPPosition(u8),
}

#[derive(ScryptoSbor, Clone)]
pub struct MarketConfig {
    pub max_cdp_position: u8,
}
impl MarketConfig {
    pub fn check(&self) -> Result<(), String> {
        if self.max_cdp_position == 0 {
            return Err("Max CDP position must be greater than 0".into());
        }

        Ok(())
    }

    pub fn update(&mut self, pool_config_input: UpdateMarketConfigInput) -> Result<(), String> {
        match pool_config_input {
            UpdateMarketConfigInput::MaxCDPPosition(max_cdp_position) => {
                self.max_cdp_position = max_cdp_position;
            }
        }

        self.check()?;

        Ok(())
    }
}
