use scrypto::prelude::*;

#[derive(ScryptoSbor, Debug, Clone, Default)]
pub struct OperatingStatusValue {
    enabled: bool,
    set_by_admin: bool,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub enum OperatingService {
    Contribute,
    Redeem,
    AddCollateral,
    RemoveCollateral,
    Borrow,
    Repay,
    Refinance,
    Liquidation,
    Flashloan,
}

#[derive(ScryptoSbor, Default)]
pub struct OperatingStatus {
    pub is_contribute_enabled: OperatingStatusValue,
    pub is_redeem_enabled: OperatingStatusValue,
    pub is_deposit_enabled: OperatingStatusValue,
    pub is_withdraw_enabled: OperatingStatusValue,
    pub is_borrow_enabled: OperatingStatusValue,
    pub is_repay_enabled: OperatingStatusValue,
    pub is_refinance_enabled: OperatingStatusValue,
    pub is_liquidate_enabled: OperatingStatusValue,
    pub is_flashloan_enabled: OperatingStatusValue,
}
//
impl OperatingStatus {
    pub fn new() -> Self {
        Self {
            is_contribute_enabled: OperatingStatusValue {
                enabled: true,
                set_by_admin: false,
            },
            is_redeem_enabled: OperatingStatusValue {
                enabled: true,
                set_by_admin: false,
            },
            is_deposit_enabled: OperatingStatusValue {
                enabled: true,
                set_by_admin: false,
            },
            is_withdraw_enabled: OperatingStatusValue {
                enabled: true,
                set_by_admin: false,
            },
            is_borrow_enabled: OperatingStatusValue {
                enabled: true,
                set_by_admin: false,
            },
            is_repay_enabled: OperatingStatusValue {
                enabled: true,
                set_by_admin: false,
            },
            is_refinance_enabled: OperatingStatusValue {
                enabled: true,
                set_by_admin: false,
            },
            is_liquidate_enabled: OperatingStatusValue {
                enabled: true,
                set_by_admin: false,
            },
            is_flashloan_enabled: OperatingStatusValue {
                enabled: true,
                set_by_admin: false,
            },
        }
    }

    pub fn update(
        &mut self,
        operating_status: OperatingService,
        enabled: bool,
        set_by_admin: bool,
    ) -> Result<(), String> {
        let field = match operating_status {
            OperatingService::Contribute => &mut self.is_contribute_enabled,
            OperatingService::Redeem => &mut self.is_redeem_enabled,
            OperatingService::AddCollateral => &mut self.is_deposit_enabled,
            OperatingService::RemoveCollateral => &mut self.is_withdraw_enabled,
            OperatingService::Borrow => &mut self.is_borrow_enabled,
            OperatingService::Repay => &mut self.is_repay_enabled,
            OperatingService::Refinance => &mut self.is_refinance_enabled,
            OperatingService::Liquidation => &mut self.is_liquidate_enabled,
            OperatingService::Flashloan => &mut self.is_flashloan_enabled,
        };

        if field.set_by_admin && !set_by_admin {
            return Err(format!(
                "The operating status for {:?} is set by admin",
                operating_status
            ));
        }

        field.enabled = enabled;
        field.set_by_admin = set_by_admin;

        Ok(())
    }

    pub fn check(&self, value: OperatingService) -> bool {
        match value {
            OperatingService::Contribute => self.is_contribute_enabled.enabled,
            OperatingService::Redeem => self.is_redeem_enabled.enabled,
            OperatingService::AddCollateral => self.is_deposit_enabled.enabled,
            OperatingService::RemoveCollateral => self.is_withdraw_enabled.enabled,
            OperatingService::Borrow => self.is_borrow_enabled.enabled,
            OperatingService::Repay => self.is_repay_enabled.enabled,
            OperatingService::Refinance => self.is_refinance_enabled.enabled,
            OperatingService::Liquidation => self.is_liquidate_enabled.enabled,
            OperatingService::Flashloan => self.is_flashloan_enabled.enabled,
        }
    }
}
