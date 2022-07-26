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
    // SetSwitch
    // BML-00-10
    #[payable]
    pub fn set_switch(&mut self, u_switch: bool, s_switch: bool) {
        assert_one_yocto();
        self.assert_owner();
        
        self.buy_u_switch = u_switch;
        self.buy_s_switch = s_switch;
        
        Event::SetSwitch{caller_id: &env::predecessor_account_id(), u_switch, s_switch}.emit();
    }
    // SetSBoxPrice
    // BML-00-15
    #[payable]
    pub fn set_sbox_price(&mut self, sbox_price: U128) {
        assert_one_yocto();
        self.assert_owner();
        
        self.sbox_price_info.sbox_price = sbox_price.0;
        
        Event::SetSBoxPrice{caller_id: &env::predecessor_account_id(), sbox_price: &sbox_price}.emit();
    }



    // setSSPTWapPriceMin
    // BML-00-19
    #[payable]
    pub fn set_tia_twap_price_min(&mut self, price: U128) {
        assert_one_yocto();
        self.assert_owner();
        
        let old = self.sbox_price_info.ssp_twap_price_min;
        self.sbox_price_info.ssp_twap_price_min = price.0;

        Event::SetSSPTWapPriceMin{caller_id: &env::predecessor_account_id(), set_tia_twap_price_min: &price}.emit();
    }
    // setRewardRate
    // BML-00-15
    #[payable]
    pub fn set_reward_rate(&mut self, ship_reward_rate: u8, risker_reward_rate: u8, bank_reward_rate: u8) {
        assert_one_yocto();
        self.assert_owner();
        require!( (ship_reward_rate+risker_reward_rate+bank_reward_rate)<=RATE_DENOMINATOR, "boxmall: error" );

        let old_ship_reward_rate = self.reward_rate.ship_reward_rate;
        let old_risker_reward_rate = self.reward_rate.risker_reward_rate;
        let old_bank_reward_rate = self.reward_rate.bank_reward_rate;
        
        self.reward_rate.ship_reward_rate = ship_reward_rate;
        self.reward_rate.risker_reward_rate = risker_reward_rate;
        self.reward_rate.bank_reward_rate = bank_reward_rate;

        //
        //emit SetRewardRate(msg.sender, oldShipRewardRate, _shipRewardRate, oldRiskerRewardRate, _riskerRewardRate, oldBankRewardRate, _bankRewardRate);
        Event::SetRewardRate{
            caller_id: &env::predecessor_account_id(),
            old_ship_reward_rate, new_ship_reward_rate: ship_reward_rate,
            old_risker_reward_rate, new_risker_reward_rate: risker_reward_rate,
            old_bank_reward_rate, new_bank_reward_rate: bank_reward_rate
        }.emit();
    }
    //
    // setUBoxSaleNumLimit
    // BML-00-11
    #[payable]
    pub fn set_ubox_sale_num_limit(&mut self, limit: u32) {
        assert_one_yocto();
        self.assert_owner();
        self.ubox_sale_num_limit = limit;
        Event::SetUBoxSaleNumLimit{caller_id: &env::predecessor_account_id(), limit}.emit();
    }
    // setNumLimit
    // BML-00-12
    #[payable]
    pub fn set_num_limit(&mut self, limit: u8) {
        assert_one_yocto();
        self.assert_owner();
        self.num_limit = limit;
        Event::SetNumLimit{caller_id: &env::predecessor_account_id(), limit}.emit();
    }

    // set_reward_accounts
    // BML-00-14
    pub fn set_reward_accounts(&mut self, accounts: String){

    }

    // addUBoxSale
    // BML-00-20
    #[payable]
    pub fn add_ubox_sale(&mut self, start: TimeStampSec, end: TimeStampSec, total: u32, price: U128) {
        assert_one_yocto();
        self.assert_owner();
        require!(start < end, "ERR_INVALID_END_TIME");
        require!(start > nano_to_sec(env::block_timestamp()), "ERR_INVALID_START_TIME");

        self.ubox_sale_pool.push(&UBoxSale{
            total,
            price: price.0,
            sale: 0,
            start,
            end
            });
    }
    // setUBoxSale
    // BML-00-21
    #[payable]
    pub fn set_ubox_sale(&mut self, index: u64, start: TimeStampSec, end: TimeStampSec, total: u32, price: U128) {
        assert_one_yocto();
        self.assert_owner();
        require!(start < end, "ERR_INVALID_END_TIME");
        require!(start > nano_to_sec(env::block_timestamp()), "ERR_INVALID_START_TIME");

        let mut ubox_sale = self.ubox_sale_pool.get(index).expect("Invalid index");
        ubox_sale.start = start;
        ubox_sale.end = end;
        ubox_sale.total = total;
        ubox_sale.price = price.0;
        self.ubox_sale_pool.replace(index, &ubox_sale);

    } 


    // setBank
    #[payable]
    pub fn set_bank(&mut self, bank: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        assert_eq!(bank, AccountId::new_unchecked("".to_string()), "boxmall: invalid address");
        self.bank = bank;
    }

    // setBankU
    #[payable]
    pub fn set_bank_u(&mut self, bank: AccountId)  {
        assert_one_yocto();
        self.assert_owner();
        assert_eq!(bank, AccountId::new_unchecked("".to_string()), "boxmall: invalid address");
        self.bank_u = bank;
    }
    
    //
    #[payable]
    pub fn set_oracle(&mut self, oracle: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        assert_eq!(oracle, AccountId::new_unchecked("".to_string()), "boxmall: invalid address");
        self.oracle = oracle;
    }

    #[payable]
    pub fn set_risker_pool(&mut self, risker_pool: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        assert_eq!(risker_pool, AccountId::new_unchecked("".to_string()), "boxmall: invalid address");
        self.risker_pool = risker_pool;
    }

    #[payable]
    pub fn set_rank_pool(&mut self, rank_pool: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        assert_eq!(rank_pool, AccountId::new_unchecked("".to_string()), "boxmall: invalid address");
        self.rank_pool = rank_pool;
    }

    #[payable]
    pub fn set_ship_pool(&mut self, ship_pool: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        assert_eq!(ship_pool, AccountId::new_unchecked("".to_string()), "boxmall: invalid address");
        self.ship_pool = ship_pool;
    }

    #[payable]
    pub fn set_luck_pool(&mut self, luck: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        assert_eq!(luck, AccountId::new_unchecked("".to_string()), "boxmall: invalid address");
        self.luck = luck;
    }

    
    // setSPriceSwitch
    // BML-00-16
    #[payable]
    pub fn set_sprice_switch(&mut self, switch: bool) {
        assert_one_yocto();
        self.assert_owner();
        self.s_price_switch = switch;
    }
}
#[near_bindgen]
impl Contract {
    // BML-00-06
    pub fn ship_wallet_add( &mut self, to: AccountId, amount: U128 ) {
        //IERC20(ssp).safeTransferFrom(msg.sender, address(this), _amount);
        let mut balance = self.balances.get(&to).unwrap_or(0);
        balance += amount.0;
        self.balances.insert(&to, &balance);
        self.total_balance_shipwallet += amount.0;
    }
    // BML-00-07
    pub fn ship_wallet_sub( &mut self, to: AccountId, amount: u128 ) {
        //IERC20(ssp).safeTransfer(msg.sender, _amount);
        let mut balance = self.balances.get(&to).unwrap();
        balance -= amount;
        self.balances.insert(&to, &balance);
        self.total_balance_shipwallet -= amount;
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