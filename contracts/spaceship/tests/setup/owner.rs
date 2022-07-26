use crate::setup::*;

impl Env {
    pub fn set_owner(
        &self,
        operator: &UserAccount,
        new_owner: &UserAccount,
        deposit: u128,
    ) -> ExecutionResult {
        operator.function_call(
            self.spaceship.contract.set_owner(new_owner.account_id()),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn mint_eng(&self, operator: &UserAccount, amount: U128) -> ExecutionResult {
        operator.function_call(self.spaceship.contract.mint_eng(amount), MAX_GAS.0, 1)
    }

    pub fn set_eng_icon(&self, operator: &UserAccount, icon: String) -> ExecutionResult {
        operator.function_call(
            self.spaceship.contract.set_eng_icon(icon.clone()),
            MAX_GAS.0,
            1,
        )
    }

    pub fn set_ship_icon(
        &self,
        operator: &UserAccount,
        ship_type_sub_type: String,
        icon: String,
    ) -> ExecutionResult {
        operator.function_call(
            self.spaceship
                .contract
                .set_ship_icon(ship_type_sub_type.clone(), icon.clone()),
            MAX_GAS.0,
            1,
        )
    }
}
