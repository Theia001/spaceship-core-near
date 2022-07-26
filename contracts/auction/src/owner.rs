use crate::*;

impl Contract {
    pub fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "ERR_NOT_ALLOWED"
        );
    }

}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn set_owner(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.owner_id = owner_id;
    }


    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have, you need to implement migration from old state 
    /// (keep the old struct with different name to deserialize it first).
    /// After migration goes live, revert back to this implementation for next updates.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let contract: Contract = env::state_read().expect("ERR_NOT_INIT");
        contract
    }

    /* ========== GOVERNANCE ========== */
    // AUC_O0_04
    pub fn add_auction_info(&mut self, token_id: TokenId, price: U128, up_price: U128, start: TimeStampSec, end: TimeStampSec) -> AuctionId {
        self.assert_owner();
        require!(start < end, "ERR_INVALID_END_TIME");
        require!(start > nano_to_sec(env::block_timestamp()), "ERR_INVALID_START_TIME");
        log!("auction.add_auction: token_id: {}, price: {}, up_price: {}, start: {}, end: {}", token_id, price.0, up_price.0, start, end);

        self.pool.push(&AuctionInfo{
            buyer : self.owner_id.clone(),
            token_id,
            price : price.0,
            up_price : up_price.0,
            start,
            end,
            team_fund : 0,
            claimed : false,
            team_fund_claimed : false
            });
        return self.pool.len()-1;
    }

    // AUC_O0_05
    pub fn update_auction_info(&mut self, auction_id: AuctionId, price: U128, up_price: U128, start: TimeStampSec, end: TimeStampSec) {
        self.assert_owner();
        let mut auction_info: AuctionInfo = self.pool.get(auction_id).expect("Invalid auction_id");
        require!(start < end, "ERR_INVALID_END_TIME");
        require!(start > nano_to_sec(env::block_timestamp()), "ERR_INVALID_START_TIME");
        log!("auction.update_auction_info: auction_id: {}, price: {}, up_price: {}, start: {}, end: {}", auction_id, price.0, up_price.0, start, end);

        auction_info.start = start;
        auction_info.end = end;
        auction_info.up_price = up_price.0;
        auction_info.price = price.0;
        self.pool.replace(auction_id, &auction_info);
        Event::UpdateAuctionInfo{ caller_id: &env::predecessor_account_id(), auction_id, price: &price, up_price: &up_price, start, end}.emit();
    }

    // AUC_O0_07
    pub fn set_team_account(&mut self, team_id: AccountId) {
        self.assert_owner();
        log!("auction.set_team_id: team_id: {}", team_id.to_string());
        self.team_id = team_id;
    }

    // AUC_O0_08
    pub fn set_extend_duration_sec(&mut self, duration_sec: u64 ) {
        self.assert_owner();
        log!("auction.set_duration_sec: duration_sec: {}", duration_sec);
        self.duration_sec = duration_sec;
    }

    // AUC_O0_06
    pub fn team_withdraw(&mut self, auction_id: AuctionId) {
        self.assert_owner();
        let mut auction_info: AuctionInfo = self.pool.get(auction_id).expect("Invalid auction_id");
        require!(!auction_info.team_fund_claimed, "Auction: already claimed");
        require!(auction_info.end + ONE_DAY_IN_SECS < nano_to_sec(env::block_timestamp()), "Auction: must be 24 hours after owner not claimed");

        ext_fungible_token::ft_transfer(
            self.team_id.clone(), 
            U128(auction_info.team_fund),
            None,
            self.token_id.clone(),
            1,
            GAS_FOR_TRANSFER 
        ); 
        auction_info.team_fund_claimed = true;
        self.pool.replace(auction_id, &auction_info);
        log!("auction.withdraw: auction_id: {}, receiver_id: {}, amount:  {}", auction_id, self.team_id.to_string(), auction_info.team_fund);
    }


}


#[cfg(target_arch = "wasm32")]
mod upgrade {
    use near_sdk::Gas;
    use near_sys as sys;

    use super::*;

    /// Gas for calling migration call.
    pub const GAS_FOR_MIGRATE_CALL: Gas = Gas(5_000_000_000_000);

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub fn upgrade() {
        env::setup_panic_hook();
        let contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_id = env::current_account_id().as_bytes().to_vec();
        let method_name = "migrate".as_bytes().to_vec();
        unsafe {
            // Load input (wasm code) into register 0.
            sys::input(0);
            // Create batch action promise for the current contract ID
            let promise_id =
                sys::promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
            // 1st action in the Tx: "deploy contract" (code is taken from register 0)
            sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // 2nd action in the Tx: call this_contract.migrate() with remaining gas
            let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_MIGRATE_CALL;
            sys::promise_batch_action_function_call(
                promise_id,
                method_name.len() as _,
                method_name.as_ptr() as _,
                0 as _,
                0 as _,
                0 as _,
                attached_gas.0,
            );
        }
    }
}