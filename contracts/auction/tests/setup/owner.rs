use crate::setup::*;

impl Env {
    pub fn set_owner(
        &self,
        operator: &UserAccount,
        new_owner: &UserAccount,
        deposit: u128,
    ) -> ExecutionResult {
        operator.function_call(
            self.auction.contract.set_owner(new_owner.account_id()),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn add_auction_info(
        &self,
        operator: &UserAccount,
        token_id: String,
        price: u128,
        up: u128,
        start: TimeStampSec,
        end: TimeStampSec,
    ) -> ExecutionResult {
        operator.function_call(
            self.auction.contract.add_auction_info(
                token_id.clone(),
                price.into(),
                up.into(),
                start,
                end,
            ),
            MAX_GAS.0,
            0,
        )
    }

    pub fn update_auction_info(
        &self,
        operator: &UserAccount,
        auction_id: AuctionId,
        price: u128,
        up: u128,
        start: TimeStampSec,
        end: TimeStampSec,
    ) -> ExecutionResult {
        operator.function_call(
            self.auction.contract.update_auction_info(
                auction_id,
                price.into(),
                up.into(),
                start,
                end,
            ),
            MAX_GAS.0,
            0,
        )
    }

    pub fn set_team_account(
        &self,
        operator: &UserAccount,
        team_id: AccountId,
    ) -> ExecutionResult {
        operator.function_call(
            self.auction.contract.set_team_account(team_id),
            MAX_GAS.0,
            0,
        )
    }

    pub fn set_extend_duration_sec(
        &self,
        operator: &UserAccount,
        sec: u64,
    ) -> ExecutionResult {
        operator.function_call(
            self.auction.contract.set_extend_duration_sec(sec),
            MAX_GAS.0,
            0,
        )
    }

    pub fn team_withdraw(
        &self,
        operator: &UserAccount,
        auction_id: AuctionId,
    ) -> ExecutionResult {
        operator.function_call(
            self.auction.contract.team_withdraw(auction_id),
            MAX_GAS.0,
            0,
        )
    }
}
