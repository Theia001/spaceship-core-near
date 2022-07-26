use crate::setup::*;

impl Env {
    pub fn eng_ft_transfer(
        &self,
        operator: &UserAccount,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> ExecutionResult {
        operator.function_call(
            self.spaceship
                .contract
                .ft_transfer(receiver_id, amount, memo),
            MAX_GAS.0,
            1,
        )
    }

    pub fn eng_ft_register(
        &self,
        operator: &UserAccount,
        account_id: Option<AccountId>,
    ) -> ExecutionResult {
        operator.function_call(
            self.spaceship
                .contract
                .storage_deposit(account_id, Some(true)),
            MAX_GAS.0,
            to_yocto("0.00125"),
        )
    }
}
