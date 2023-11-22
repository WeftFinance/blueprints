use crate::modules::utils::is_valid_rate;
use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub enum UpdatePoolConfigInput {
    ProtocolFeeRate(Decimal),
    FlashloanFeeRate(Decimal),

    DepositLimit(Option<Decimal>),
    BorrowLimit(Option<Decimal>),
    UtilizationLimit(Option<Decimal>),

    AssetType(u8),
    LiquidationBonusRate(Decimal),
    LoanCloseFactor(Decimal),

    InterestUpdatePeriod(i64),
    PriceUpdatePeriod(i64),
    PriceExpirationPeriod(i64),
}

pub enum CheckPoolConfigLimitInput {
    DepositLimit(Decimal),
    BorrowLimit(Decimal),
    UtilizationLimit(Decimal),
}

#[derive(ScryptoSbor, Clone)]
pub struct PoolConfig {
    pub protocol_fee_rate: Decimal,
    pub flashloan_fee_rate: Decimal,

    pub asset_type: u8,

    pub liquidation_bonus_rate: Decimal,
    pub loan_close_factor: Decimal,

    pub deposit_limit: Option<Decimal>,
    pub borrow_limit: Option<Decimal>,
    pub utilization_limit: Option<Decimal>,

    pub interest_update_period: i64,
    pub price_update_period: i64,
    pub price_expiration_period: i64,
}
impl PoolConfig {
    pub fn check(&self) -> Result<(), String> {
        if !is_valid_rate(self.protocol_fee_rate) {
            return Err("Lending fee rate must be between 0 and 1".into());
        }

        if !is_valid_rate(self.flashloan_fee_rate) {
            return Err("Flashloan fee rate must be between 0 and 1".into());
        }

        if self.deposit_limit.is_some() && self.deposit_limit.unwrap() < dec!(0) {
            return Err("Deposit limit must be positive".into());
        }

        if self.borrow_limit.is_some() && self.borrow_limit.unwrap() < dec!(0) {
            return Err("Borrow limit must be positive".into());
        }

        if self.utilization_limit.is_some() && !is_valid_rate(self.utilization_limit.unwrap()) {
            return Err("Utilization limit must be between 0 and 1".into());
        }

        if !is_valid_rate(self.liquidation_bonus_rate) {
            return Err("Liquidation bonus rate must be between 0 and 1".into());
        }

        if !is_valid_rate(self.loan_close_factor) {
            return Err("Loan close factor must be between 0 and 1".into());
        }

        if self.price_update_period <= 0 {
            return Err("Price update period must be greater than 0".into());
        }

        if self.price_expiration_period <= 0 {
            return Err("Price expiration period must be greater than 0".into());
        }

        if self.price_expiration_period <= self.price_update_period {
            return Err("Price expiration period must be greater than price update period".into());
        }

        if self.interest_update_period <= 0 {
            return Err("Interest update period must be greater than 0".into());
        }

        Ok(())
    }

    pub fn update(&mut self, pool_config_input: UpdatePoolConfigInput) -> Result<(), String> {
        match pool_config_input {
            UpdatePoolConfigInput::DepositLimit(deposit_limit) => {
                if deposit_limit.is_some() && deposit_limit.unwrap() < dec!(0) {
                    return Err("Deposit limit must be positive".into());
                }

                self.deposit_limit = deposit_limit;
            }

            UpdatePoolConfigInput::BorrowLimit(borrow_limit) => {
                if borrow_limit.is_some() && borrow_limit.unwrap() < dec!(0) {
                    return Err("Borrow limit must be positive".into());
                }

                self.borrow_limit = borrow_limit;
            }

            UpdatePoolConfigInput::UtilizationLimit(utilization_limit) => {
                if utilization_limit.is_some() && utilization_limit.unwrap() < dec!(0) {
                    return Err("Utilization limit must be between 0 and 1".into());
                }

                self.utilization_limit = utilization_limit;
            }

            UpdatePoolConfigInput::FlashloanFeeRate(flashloan_fee_rate) => {
                if !is_valid_rate(flashloan_fee_rate) {
                    return Err("Flashloan fee rate must be between 0 and 1".into());
                }

                self.flashloan_fee_rate = flashloan_fee_rate;
            }

            UpdatePoolConfigInput::ProtocolFeeRate(lending_fee_rate) => {
                if !is_valid_rate(lending_fee_rate) {
                    return Err("Lending fee rate must be between 0 and 1".into());
                }

                self.protocol_fee_rate = lending_fee_rate;
            }

            UpdatePoolConfigInput::LiquidationBonusRate(liquidation_bonus_rate) => {
                if !is_valid_rate(liquidation_bonus_rate) {
                    return Err("Liquidation bonus rate must be between 0 and 1".into());
                }

                self.liquidation_bonus_rate = liquidation_bonus_rate;
            }

            UpdatePoolConfigInput::LoanCloseFactor(loan_close_factor) => {
                if !is_valid_rate(loan_close_factor) {
                    return Err("Loan close factor must be between 0 and 1".into());
                }

                self.loan_close_factor = loan_close_factor;
            }

            UpdatePoolConfigInput::AssetType(asset_type) => {
                self.asset_type = asset_type;
            }

            UpdatePoolConfigInput::InterestUpdatePeriod(interest_update_period) => {
                if interest_update_period <= 0 {
                    return Err("Interest update period must be greater than 0".into());
                }

                self.interest_update_period = interest_update_period;
            }

            UpdatePoolConfigInput::PriceUpdatePeriod(price_update_period) => {
                if price_update_period <= 0 {
                    return Err("Price update period must be greater than 0".into());
                }

                self.price_update_period = price_update_period;
            }

            UpdatePoolConfigInput::PriceExpirationPeriod(price_expiration_period) => {
                if price_expiration_period <= 0 {
                    return Err("Price expiration period must be greater than 0".into());
                }

                if price_expiration_period <= self.price_update_period {
                    return Err(
                        "Price expiration period must be greater than price update period".into(),
                    );
                }

                self.price_expiration_period = price_expiration_period;
            }
        };

        Ok(())
    }

    pub fn check_limit(&self, input: CheckPoolConfigLimitInput) -> Result<(), String> {
        match input {
            CheckPoolConfigLimitInput::DepositLimit(current_deposit) => {
                if let Some(limit) = self.deposit_limit {
                    if current_deposit > limit {
                        return Err(
                            "Deposit limit reached. Please try again with a smaller amount.".into(),
                        );
                    }
                }
            }

            CheckPoolConfigLimitInput::BorrowLimit(current_borrow) => {
                if let Some(limit) = self.borrow_limit {
                    if current_borrow > limit {
                        return Err(
                            "Borrow limit reached. Please try again with a smaller amount.".into(),
                        );
                    }
                }
            }

            CheckPoolConfigLimitInput::UtilizationLimit(current_utilization) => {
                if let Some(limit) = self.utilization_limit {
                    if current_utilization > limit {
                        return Err(
                            "Utilization limit reached. Please try again with a smaller amount."
                                .into(),
                        );
                    }
                }
            }
        };

        Ok(())
    }
}
