use crate::setup::*;

impl Env {
    pub fn mint_nft(
        &self,
        token_id: String,
    ) -> ExecutionResult {
        self.owner.function_call(
            self.nft.contract.mint(self.owner.account_id(), token_id),
            MAX_GAS.0,
            0,
        )
    }

    pub fn nft_owner(&self, token_id: TokenId) -> AccountId {
        let token: Option<Token> = self.owner
            .view(
                self.nft.account_id(),
                "nft_token",
                &json!({
                    "token_id": token_id
                }).to_string().into_bytes()
            ).unwrap_json();
        token.unwrap().owner_id
    }

}