use crate::setup::*;

impl Env {

    pub fn claim(
        &self,
        operator: &UserAccount,
        auction_id: AuctionId,
    ) -> ExecutionResult {
        operator.function_call(
            self.auction.contract.claim(auction_id),
            MAX_GAS.0,
            0,
        )
    }

    pub fn usn_bid(
        &self,
        user: &UserAccount,
        auction_id: String,
        amount: Balance,
    ) -> ExecutionResult {
        user.call(
            self.usn.account_id(),
            "ft_transfer_call",
            &json!({
                "receiver_id": self.auction.account_id(),
                "amount": U128::from(amount),
                "msg": auction_id,
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }

    pub fn ft_bid(
        &self,
        user: &UserAccount,
        auction_id: String,
        amount: Balance,
    ) -> ExecutionResult {
        user.call(
            self.ft.account_id(),
            "ft_transfer_call",
            &json!({
                "receiver_id": self.auction.account_id(),
                "amount": U128::from(amount),
                "msg": auction_id,
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }

}