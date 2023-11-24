use super::utils::is_valid_rate;
use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone, Debug)]
pub struct ISInputBreakPoint {
    pub usage: Decimal,
    pub slop: Decimal,
}

#[derive(ScryptoSbor, Default, Clone, Debug)]
pub struct ISInternalBreakPoint {
    usage: Decimal, // x
    rate: Decimal,  // y
    slop: Decimal,
}

#[derive(ScryptoSbor, Default, Clone)]
pub struct InterestStrategy {
    break_points: Vec<ISInternalBreakPoint>,
}

impl InterestStrategy {
    pub fn new() -> Self {
        Self {
            break_points: Vec::new(),
        }
    }

    pub fn set_breakpoints(
        &mut self,
        initial_rate: Decimal,
        mut input_break_points: Vec<ISInputBreakPoint>,
    ) -> Result<(), String> {
        if input_break_points.is_empty() {
            return Err("The break points vector must contain at least one element".into());
        }

        if initial_rate < dec!(0) {
            return Err("The initial rate must be greater than or equal to 0".into());
        }

        input_break_points.sort_by(|a, b| a.usage.partial_cmp(&b.usage).unwrap());

        if input_break_points[0].usage != dec!(0) {
            return Err("The first break point usage must be 0".into());
        }

        let mut break_points = Vec::new();

        break_points.push(ISInternalBreakPoint {
            usage: input_break_points[0].usage,
            rate: initial_rate,
            slop: input_break_points[0].slop,
        });

        for i in 1..input_break_points.len() {
            if !is_valid_rate(input_break_points[i].usage) {
                return Err("Usage must be between 0 and 1, inclusive".into());
            }

            if input_break_points[i - 1].usage >= input_break_points[i].usage {
                return Err("Usage must be monotonically increasing".into());
            }

            if input_break_points[i].slop < dec!(0) {
                return Err("Slop must be greater than or equal to 0".into());
            }

            let previous_bp = &break_points[i - 1];
            break_points.push(ISInternalBreakPoint {
                usage: input_break_points[i].usage,
                rate: previous_bp.rate
                    + previous_bp.slop * (input_break_points[i].usage - previous_bp.usage),
                slop: input_break_points[i].slop,
            });
        }

        self.break_points = break_points;

        Ok(())
    }

    pub fn get_interest_rate(&self, usage: Decimal) -> Result<Decimal, String> {
        if !is_valid_rate(usage) {
            return Err("Usage must be between 0 and 1, inclusive".into());
        }

        let len = self.break_points.len();
        let mut j = len - 1;

        for i in 0..len - 1 {
            if self.break_points[i].usage <= usage && usage < self.break_points[i + 1].usage {
                j = i;
                break;
            }
        }

        let break_point = &self.break_points[j];

        let interest_rate = break_point.rate + ((usage - break_point.usage) * break_point.slop);

        Ok(interest_rate)
    }
}
