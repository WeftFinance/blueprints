use scrypto::prelude::*;

#[derive(ScryptoSbor, Debug, Clone)]
pub enum OperatingStatusInput {
    Contribute(bool),
    Redeem(bool),
    Deposit(bool),
    Withdraw(bool),
    Borrow(bool),
    Repay(bool),
    Refinance(bool),
    Liquidation(bool),
    Flashloan(bool),
}

#[derive(ScryptoSbor)]
pub struct OperatingStatus {
    pub is_contribute_enabled: bool,
    pub is_redeem_enabled: bool,
    pub is_deposit_enabled: bool,
    pub is_withdraw_enabled: bool,
    pub is_borrow_enabled: bool,
    pub is_repay_enabled: bool,
    pub is_refinance_enabled: bool,
    pub is_liquidate_enabled: bool,
    pub is_flashloan_enabled: bool,
}

impl OperatingStatus {
    pub fn default() -> Self {
        Self {
            is_contribute_enabled: true,
            is_redeem_enabled: true,
            is_deposit_enabled: true,
            is_withdraw_enabled: true,
            is_borrow_enabled: true,
            is_repay_enabled: true,
            is_refinance_enabled: true,
            is_liquidate_enabled: true,
            is_flashloan_enabled: true,
        }
    }

    pub fn update(&mut self, value: OperatingStatusInput) {
        match value {
            OperatingStatusInput::Contribute(v) => self.is_contribute_enabled = v,
            OperatingStatusInput::Redeem(v) => self.is_redeem_enabled = v,
            OperatingStatusInput::Deposit(v) => self.is_deposit_enabled = v,
            OperatingStatusInput::Withdraw(v) => self.is_withdraw_enabled = v,
            OperatingStatusInput::Borrow(v) => self.is_borrow_enabled = v,
            OperatingStatusInput::Repay(v) => self.is_repay_enabled = v,
            OperatingStatusInput::Refinance(v) => self.is_refinance_enabled = v,
            OperatingStatusInput::Liquidation(v) => self.is_liquidate_enabled = v,
            OperatingStatusInput::Flashloan(v) => self.is_flashloan_enabled = v,
        }
    }

    pub fn check(&self, value: OperatingStatusInput) -> bool {
        match value {
            OperatingStatusInput::Contribute(v) => self.is_contribute_enabled == v,
            OperatingStatusInput::Redeem(v) => self.is_redeem_enabled == v,
            OperatingStatusInput::Deposit(v) => self.is_deposit_enabled == v,
            OperatingStatusInput::Withdraw(v) => self.is_withdraw_enabled == v,
            OperatingStatusInput::Borrow(v) => self.is_borrow_enabled == v,
            OperatingStatusInput::Repay(v) => self.is_repay_enabled == v,
            OperatingStatusInput::Refinance(v) => self.is_refinance_enabled == v,
            OperatingStatusInput::Liquidation(v) => self.is_liquidate_enabled == v,
            OperatingStatusInput::Flashloan(v) => self.is_flashloan_enabled == v,
        }
    }
}
