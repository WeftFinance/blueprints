use scrypto::prelude::*;

// CONSTANTS

pub const SECOND_PER_MINUTE: i64 = 60;
pub const MINUTE_PER_YEAR: i64 = 60 * 24 * 365;

// Check if the given rate is between 0 and 1
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

pub fn get_price(
    price_feed: Global<AnyComponent>,
    res_address: ResourceAddress,
) -> Result<PriceInfo, String> {
    // Bypass price feed for XRD, return 1 as XRD is the base currency
    if res_address == XRD {
        return Ok(PriceInfo {
            timestamp: Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch,
            price: dec!(1),
        });
    }

    match price_feed.call_raw::<Option<PriceInfo>>("get_price", scrypto_args!(res_address)) {
        Some(price_info) => Ok(price_info),
        None => Err("Price not found".to_string()),
    }
}
