use crate::setup::*;

impl Env {

    pub fn mint_usn(
        &self,
        user_account: &UserAccount,
        amount: u128,
    ) -> ExecutionResult {
        user_account.function_call(
            self.usn.contract.mint(amount.into()),
            MAX_GAS.0,
            1,
        )
    }

    pub fn mint_ft(
        &self,
        user_account: &UserAccount,
        amount: u128,
    ) -> ExecutionResult {
        user_account.function_call(
            self.ft.contract.mint(amount.into()),
            MAX_GAS.0,
            1,
        )
    }

    pub fn usn_balance(&self, user_account: &UserAccount) -> u128 {
        let amount: U128 = self.owner
            .view(
                self.usn.account_id(),
                "ft_balance_of",
                &json!({
                    "account_id": user_account.account_id()
                }).to_string().into_bytes()
            ).unwrap_json();
        amount.0
    }
}