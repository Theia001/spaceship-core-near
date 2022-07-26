use crate::setup::*;

impl Env {
    pub fn batch_mint(
        &self,
        operator: &UserAccount,
        owner_id: AccountId,
        ship_types: Vec<String>,
        ship_sub_types: Vec<String>,
    ) -> ExecutionResult {
        operator.function_call(
            self.spaceship
                .contract
                .batch_mint(owner_id, ship_types, ship_sub_types),
            MAX_GAS.0,
            0,
        )
    }

    pub fn nft_transfer(
        &self,
        operator: &UserAccount,
        receiver_id: AccountId,
        token_id: TokenId,
    ) -> ExecutionResult {
        operator.function_call(
            self.spaceship.contract.nft_transfer(receiver_id, token_id, None, None),
            MAX_GAS.0,
            1,
        )
    }

    pub fn nft_transfer_call(
        &self,
        operator: &UserAccount,
        receiver_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> ExecutionResult {
        operator.function_call(
            self.spaceship.contract.nft_transfer_call(receiver_id, token_id, None, None, msg),
            MAX_GAS.0,
            1,
        )
    }

    pub fn nft_payout(
        &self,
        operator: &UserAccount,
        receiver_id: AccountId,
        token_id: TokenId,
    ) -> ExecutionResult {
        operator.function_call(
            self.spaceship.contract.nft_payout(receiver_id, token_id),
            MAX_GAS.0,
            0,
        )
    }

    pub fn user_burn(&self, operator: &UserAccount, token_id: TokenId) -> ExecutionResult {
        operator.function_call(self.spaceship.contract.user_burn(token_id), MAX_GAS.0, 1)
    }

    pub fn upgrade_spaceship(
        &self,
        operator: &UserAccount,
        owner_id: AccountId,
        token_id_1: TokenId,
        token_id_2: TokenId,
        target_sub_type: u8,
        eng_amount: U128,
    ) -> ExecutionResult {
        operator.function_call(
            self.spaceship.contract.upgrade_spaceship(
                owner_id,
                token_id_1,
                token_id_2,
                target_sub_type,
                eng_amount,
            ),
            MAX_GAS.0,
            1,
        )
    }
}
