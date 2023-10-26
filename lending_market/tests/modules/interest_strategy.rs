use lending_market::modules::interest_strategy::*;
use scrypto::*;
use scrypto_test::prelude::*;

#[test]
fn test_interest_strategy_check() {
    let breakpoints_with_errors_1 = vec![
        ISInputBreakPoint {
            usage: dec!(1),
            slop: dec!(1),
        },
        ISInputBreakPoint {
            usage: dec!(0.5),
            slop: dec!(3),
        },
        ISInputBreakPoint {
            usage: dec!(0.7),
            slop: dec!(5),
        },
    ];
    let breakpoints_with_errors_2 = vec![
        ISInputBreakPoint {
            usage: dec!(0),
            slop: dec!(1),
        },
        ISInputBreakPoint {
            usage: dec!(0.5),
            slop: dec!(3),
        },
        ISInputBreakPoint {
            usage: dec!(0.7),
            slop: dec!(-5),
        },
    ];
    let breakpoints_with_errors_3 = vec![
        ISInputBreakPoint {
            usage: dec!(0),
            slop: dec!(1),
        },
        ISInputBreakPoint {
            usage: dec!(1.5),
            slop: dec!(3),
        },
        ISInputBreakPoint {
            usage: dec!(0.5),
            slop: dec!(5),
        },
    ];

    let breakpoints_with_errors_4 = vec![
        ISInputBreakPoint {
            usage: dec!(0),
            slop: dec!(1),
        },
        ISInputBreakPoint {
            usage: dec!(0.5),
            slop: dec!(3),
        },
        ISInputBreakPoint {
            usage: dec!(0.5),
            slop: dec!(5),
        },
    ];
    assert!(InterestStrategy::new()
        .set_breakpoints(dec!(0.05), breakpoints_with_errors_1)
        .is_err());

    assert!(InterestStrategy::new()
        .set_breakpoints(dec!(0.05), breakpoints_with_errors_2)
        .is_err());

    assert!(InterestStrategy::new()
        .set_breakpoints(dec!(0.05), breakpoints_with_errors_3)
        .is_err());

    assert!(InterestStrategy::new()
        .set_breakpoints(dec!(0.05), breakpoints_with_errors_4)
        .is_err());
}

#[test]
fn test_interest_strategy_check_get_rate() {
    let mut interest_strategy = InterestStrategy::new();

    let breakpoints = vec![
        ISInputBreakPoint {
            usage: dec!(0),
            slop: dec!(1),
        },
        ISInputBreakPoint {
            usage: dec!(0.5),
            slop: dec!(3),
        },
        ISInputBreakPoint {
            usage: dec!(0.7),
            slop: dec!(5),
        },
    ];

    let result = interest_strategy.set_breakpoints(dec!(0.05), breakpoints);

    assert!(result.is_ok());

    assert!(interest_strategy.get_interest_rate(dec!(-0.1)).is_err());

    assert!(interest_strategy.get_interest_rate(dec!(1.1)).is_err());
}

#[test]
fn test_interest_strategy_get_rate_value() {
    let mut interest_strategy = InterestStrategy::new();

    let breakpoints = vec![
        ISInputBreakPoint {
            usage: dec!(0),
            slop: dec!(1),
        },
        ISInputBreakPoint {
            usage: dec!(0.5),
            slop: dec!(3),
        },
        ISInputBreakPoint {
            usage: dec!(0.7),
            slop: dec!(5),
        },
    ];

    let result = interest_strategy.set_breakpoints(dec!(0.05), breakpoints);

    assert!(result.is_ok());

    assert_eq!(
        interest_strategy.get_interest_rate(dec!(0.0)),
        Ok(dec!(0.05))
    );
    assert_eq!(
        interest_strategy.get_interest_rate(dec!(0.5)),
        Ok(dec!(0.55))
    );
    assert_eq!(
        interest_strategy.get_interest_rate(dec!(0.7)),
        Ok(dec!(1.15))
    );
    assert_eq!(interest_strategy.get_interest_rate(dec!(1)), Ok(dec!(2.65)));

    assert!(interest_strategy.get_interest_rate(dec!(-0.1)).is_err());

    assert!(interest_strategy.get_interest_rate(dec!(1.1)).is_err());
}
