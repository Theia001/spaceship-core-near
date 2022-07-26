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
    #[private]
    pub fn update_decay_table(&mut self, per_block_reward: u128) {
        self.internal_update_reward("00".parse().unwrap());

        let period: u32 = self.phase(self.get_last_block_applicable()) as u32;
        let mut decay: u128 = 0;

        for i in period..self.total_period{
            if i == period {
                decay = per_block_reward;
            }
            else {
                decay = self.decay_table[(i-1) as usize]*(self.format_rate as u128 - self.decay_rate as u128)/self.format_rate as u128;
            }
            self.decay_table[i as usize] = decay;
        }
    }
    #[private]
    pub fn update_slot_enable(&mut self, slot_index: u64, price: u128, enable: bool) {
        let mut slot_node = self.slot.get(slot_index).expect("Invalid slot_index");
        slot_node.price = price;
        slot_node.enable = enable;
        self.slot.replace(slot_index,&slot_node);
    }
    #[private]
    pub fn withdraw_wrong_token(&mut self, token: AccountId, from: AccountId, amount: u128) {
        ext_fungible_token::ft_transfer(
            from.clone(), 
            U128(amount), 
            None, 
            token.clone(), 
            0, 
            GAS_FOR_TRANSFER
        );
    }
    #[private]
    pub fn set_burn(&mut self, burned_addr: AccountId) {
        self.assert_owner();
        require!(burned_addr != "00".parse().unwrap(), "invalid address");
        self.burned_addr = burned_addr;
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