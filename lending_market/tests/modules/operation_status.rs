use lending_market::modules::operation_status::*;

macro_rules! test_update_operating_status {
    ($operating_status:expr, $input:expr) => {
        $operating_status.update($input, false, false).unwrap();
        assert_eq!($operating_status.check($input), false);

        $operating_status.update($input, true, true).unwrap();
        assert_eq!($operating_status.check($input), true);
    };
}

macro_rules! test_update_operating_status_error {
    ($operating_status:expr, $input:expr) => {
        $operating_status.update($input, false, false).unwrap();
        assert_eq!($operating_status.check($input), false);

        $operating_status.update($input, true, true).unwrap();
        assert_eq!($operating_status.check($input), true);

        $operating_status
            .update($input, false, false)
            .expect_err("Should return an error");
    };
}

#[test]
fn test_update_operating_status() {
    let mut operating_status = OperatingStatus::new();

    test_update_operating_status!(operating_status, OperatingService::Contribute);
    test_update_operating_status!(operating_status, OperatingService::Redeem);
    test_update_operating_status!(operating_status, OperatingService::AddCollateral);
    test_update_operating_status!(operating_status, OperatingService::RemoveCollateral);
    test_update_operating_status!(operating_status, OperatingService::Borrow);
    test_update_operating_status!(operating_status, OperatingService::Repay);
    test_update_operating_status!(operating_status, OperatingService::Refinance);
    test_update_operating_status!(operating_status, OperatingService::Liquidation);
    test_update_operating_status!(operating_status, OperatingService::Flashloan);
}

#[test]
fn test_update_operating_status_error() {
    let mut operating_status = OperatingStatus::new();

    test_update_operating_status_error!(operating_status, OperatingService::Contribute);
    test_update_operating_status_error!(operating_status, OperatingService::Redeem);
    test_update_operating_status_error!(operating_status, OperatingService::AddCollateral);
    test_update_operating_status_error!(operating_status, OperatingService::RemoveCollateral);
    test_update_operating_status_error!(operating_status, OperatingService::Borrow);
    test_update_operating_status_error!(operating_status, OperatingService::Repay);
    test_update_operating_status_error!(operating_status, OperatingService::Refinance);
    test_update_operating_status_error!(operating_status, OperatingService::Liquidation);
    test_update_operating_status_error!(operating_status, OperatingService::Flashloan);
}
