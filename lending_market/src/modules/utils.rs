use scrypto::prelude::*;

/// Check if the given rate is between 0 and 1
pub fn is_valid_rate(rate: Decimal) -> bool {
    rate >= dec!(0) && rate <= dec!(1)
}

#[derive(ScryptoSbor, PartialEq)]
pub enum WithdrawType {
    TemporaryUse,
    LiquidityWithdrawal,
}

#[derive(ScryptoSbor, PartialEq)]
pub enum DepositType {
    FromTemporaryUse,
    LiquiditySupply,
}

#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
    pub timestamp: i64,
    pub price: Decimal,
}
