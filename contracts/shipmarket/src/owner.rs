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

    #[payable]
    pub fn set_buy_fee_rate(&mut self, fee_rate: u8) {
        assert_one_yocto();
        self.assert_owner();
        self.fee_rate = fee_rate;
    }

    #[payable]
    pub fn set_target_fee(&mut self, ship_type: u8, ssp_fee: U128, eng_fee: U128) {
        assert_one_yocto();
        self.assert_owner();
        self.target_mint_fee.insert(&ship_type, &vec![ssp_fee.0, eng_fee.0]);
    }

    #[payable]
    pub fn set_no_target_fee(&mut self, ship_type: u8, ssp_fee: U128, eng_fee: U128) {
        assert_one_yocto();
        self.assert_owner();
        self.no_target_mint_fee.insert(&ship_type, &vec![ssp_fee.0, eng_fee.0]);
    }

   // setBank
   #[payable]
   pub fn set_bank(&mut self, bank: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        assert_eq!(bank, AccountId::new_unchecked("".to_string()), "boxmall: invalid address");
        self.bank = bank;
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
        self.luck_pool = luck;
    }
}