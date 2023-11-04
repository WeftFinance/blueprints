use crate::lending_market::lending_market::*;
use crate::modules::utils::*;
use crate::modules::{interest_strategy::*, liquidation_threshold::*, pool_config::*};
use scrypto::blueprints::consensus_manager::*;
use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct LendingPoolState {
    /// Global pool component holding all the liquidity
    pub pool: Global<SingleResourcePool>,

    /// Vaults holding pool units locked as collateral
    pub collaterals: Vault,

    /// Reserve retention collected by the the protocol
    pub reserve: Vault,

    ///
    pub pool_res_address: ResourceAddress,

    ///* State *///

    ///
    pub last_price: Decimal,

    ///
    pub price_updated_at: i64,

    ///* Loan State *///

    ///
    pub total_loan: Decimal,

    ///
    pub total_loan_unit: Decimal,

    ///
    pub interest_rate: Decimal,

    ///
    pub interest_updated_at: i64,

    ///* Configs *///

    ///
    pub price_feed_comp: Global<AnyComponent>,

    ///
    pub interest_strategy: InterestStrategy,

    ///
    pub liquidation_threshold: LiquidationThreshold,

    ///
    pub pool_config: PoolConfig,
}

impl LendingPoolState {
    ///* CONFIG METHODS *///

    pub fn set_price_feed(&mut self, price_feed: Global<AnyComponent>) {
        self.price_feed_comp = price_feed;
    }

    pub fn update_config(&mut self, value: UpdatePoolConfigInput) -> Result<(), String> {
        self.pool_config.update_config(value)
    }

    pub fn set_interest_strategy(
        &mut self,
        initial_rate: Decimal,
        interest_options_break_points: Vec<ISInputBreakPoint>,
    ) -> Result<(), String> {
        self.interest_strategy
            .set_breakpoints(initial_rate, interest_options_break_points)
    }

    pub fn update_liquidation_threshold(
        &mut self,
        value: UpdateLiquidationThresholdInput,
    ) -> Result<(), String> {
        self.liquidation_threshold
            .update_liquidation_threshold(value)
    }

    /// Get the current loan unit ratio ///

    pub fn get_loan_unit_ratio(&mut self) -> Result<PreciseDecimal, String> {
        // convert total_loan_unit and total_loan to PreciseDecimal to improve precision and reduce rounding errors
        let ratio = if self.total_loan != 0.into() {
            PreciseDecimal::from(self.total_loan_unit) / PreciseDecimal::from(self.total_loan)
        } else {
            1.into()
        };

        if ratio > 1.into() {
            return Err("Loan unit ratio cannot be greater than 1".to_string());
        }

        Ok(ratio)
    }

    ///* CORE LOGIC AND UTILITY METHODS *///

    pub fn contribute_proxy(&self, assets: Bucket) -> Result<Bucket, String> {
        let amout = assets.amount();

        let (pool_available_amount, pool_borrowed_amount) = self.pool.get_pooled_amount();

        // Check if the deposit limit is reached
        self.pool_config
            .check_limit(CheckPoolConfigLimitInput::DepositLimit(
                pool_available_amount + pool_borrowed_amount + amout,
            ))?;

        Ok(self.pool.contribute(assets))
    }

    pub fn redeem_proxy(&self, assets: Bucket) -> Bucket {
        self.pool.redeem(assets)
    }

    pub fn add_pool_units_as_collateral(&mut self, pool_units: Bucket) -> Result<(), String> {
        if pool_units.amount() == 0.into() {
            return Ok(());
        }

        if pool_units.resource_address() != self.collaterals.resource_address() {
            return Err("Pool unit resource address missmatch".into());
        }

        self.collaterals.put(pool_units);

        Ok(())
    }

    pub fn remove_pool_units_from_collateral(
        &mut self,
        pool_unit_amount: Decimal,
    ) -> Result<Bucket, String> {
        if pool_unit_amount == 0.into() {
            return Err("Pool unit amount must be positive".into());
        }

        if pool_unit_amount > self.collaterals.amount() {
            return Err("Not enough pool units to remove from collateral".into());
        }

        Ok(self.collaterals.take_advanced(
            pool_unit_amount,
            WithdrawStrategy::Rounded(RoundingMode::ToZero),
        ))
    }

    /// Handle request to increse borrowed amount.
    /// it remove requested liquidity and updated the pool loan state based on input interest startegy
    pub fn withdraw_for_borrow(&mut self, amount: Decimal) -> Result<(Bucket, Decimal), String> {
        if amount == 0.into() {
            return Err("Amount must be positive".into());
        }

        let (pool_available_amount, pool_borrowed_amount) = self.pool.get_pooled_amount();

        // Check if the borrow limit is reached
        self.pool_config
            .check_limit(CheckPoolConfigLimitInput::BorrowLimit(
                pool_borrowed_amount + amount,
            ))?;

        // Check if utilization rate is not exceeded

        self.pool_config
            .check_limit(CheckPoolConfigLimitInput::UtilizationLimit(
                (pool_borrowed_amount + amount)
                    / ((pool_available_amount + pool_borrowed_amount) + amount),
            ))?;

        let loan_unit = self._update_loan_unit(amount)?;

        let result = (
            self.pool.protected_withdraw(
                amount,
                WithdrawType::TemporaryUse,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            ),
            loan_unit,
        );

        Ok(result)
    }

    /// Handle request to decrease borrowed amount.
    /// it add back liquidity and updated the pool loan state based on input interest startegy
    pub fn deposit_for_repay(&mut self, payment: Bucket) -> Result<Decimal, String> {
        if payment.resource_address() != self.pool_res_address {
            return Err("Payment resource address missmatch".into());
        }

        let loan_unit = self._update_loan_unit(-payment.amount())?;

        self.pool
            .protected_deposit(payment, DepositType::FromTemporaryUse);

        // returned unit should be negative or 0
        // Send back positive loan_unit to evoid confusion at higher level in the stack
        Ok(-loan_unit)
    }

    pub fn update_interest_and_price(&mut self) -> Result<(), String> {
        /* DEBONCING INTEREST DOWN TO 1 MINUTES */

        let before = self.interest_updated_at;
        let now: i64 = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;

        let period_in_minute = (now - before) / 60;

        if period_in_minute <= 1 {
            return Ok(());
        }

        /* UPDATING PRICE */

        // Debounce price update to configured period (in minutes)
        if period_in_minute >= self.pool_config.price_update_period {
            // TODO: Handle XRD price update
            // if self.pool_res_address == XRD {
            //     self.last_price = dec!(1);
            //     self.price_updated_at = now;
            //     return Ok(());
            // }

            let price_feed_result = self
                .price_feed_comp
                .call_raw::<Option<PriceInfo>>("get_price", scrypto_args!(self.pool_res_address));

            if price_feed_result.is_none() {
                return Err("Price feed returned None".to_string());
            }

            let price_feed_result = price_feed_result.unwrap();

            // TODO: Handle price update too old
            // if (now - price_feed_result.timestamp) >= self.pool_config.price_update_period {
            //     return Err("Price feed is too old".to_string());
            // }

            self.price_updated_at = now;
            self.last_price = price_feed_result.price;
        }

        /* UPDATING INTEREST RATE */

        let (pool_available_amount, pool_borrowed_amount) = self.pool.get_pooled_amount();

        let pool_total_liquidity = pool_available_amount + pool_borrowed_amount;

        let pool_utilization = if pool_total_liquidity == 0.into() {
            Decimal::ZERO
        } else {
            pool_borrowed_amount / pool_total_liquidity
        };

        self.interest_updated_at = now;

        self.interest_rate = self.interest_strategy.get_interest_rate(pool_utilization)?;

        let minute_interest_rate = Decimal::ONE + (self.interest_rate / dec!(525600));

        let new_total_loan_amount =
            self.total_loan * minute_interest_rate.checked_powi(period_in_minute).unwrap();

        let accrued_interest = new_total_loan_amount - self.total_loan;

        self.total_loan = new_total_loan_amount;

        // Increase pool liquidity

        self.pool.increase_external_liquidity(accrued_interest);

        // Collect protocol fees on accrued interest
        let protocol_fees = accrued_interest * self.pool_config.lending_fee_rate;

        self.reserve.put(self.pool.protected_withdraw(
            protocol_fees,
            WithdrawType::LiquidityWithdrawal,
            WithdrawStrategy::Rounded(RoundingMode::ToZero),
        ));

        Ok(())
    }

    ///* CORE LOGIC AND UTILITY METHODS: PRIVATE *///

    fn _update_loan_unit(&mut self, amount: Decimal) -> Result<Decimal, String> {
        let unit_ratio = self.get_loan_unit_ratio()?;

        let unit = (amount * unit_ratio) //
            .checked_truncate(RoundingMode::ToZero)
            .unwrap();

        self.total_loan += amount;

        self.total_loan_unit += unit;

        // TODO: Better rounding strategy
        if amount <= 0.into() {
            // Rounding error appears on repay
            self.total_loan_unit = self
                .total_loan_unit
                .checked_round(17, RoundingMode::ToZero)
                .unwrap();
        }
        // if self.total_loan == 0.into() {
        //     self.total_loan_unit = 0.into();
        // }
        // if self.total_loan_unit == 0.into() {
        //     self.total_loan = 0.into();
        // }

        if self.total_loan_unit < 0.into() {
            return Err("Total loan unit cannot be negative".to_string());
        }

        if self.total_loan < 0.into() {
            return Err("Total loan cannot be negative".to_string());
        }

        Ok(unit)
    }
}
