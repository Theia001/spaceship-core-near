use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub magicbox: AccountId,
    pub usn: AccountId,
    pub token_tia: AccountId,
    pub bank: AccountId,
    pub bank_u: AccountId,
    pub oracle: AccountId,
    pub risker_pool: AccountId,
    pub rank_pool: AccountId,
    pub ship_pool: AccountId,
    pub luck: AccountId,

    pub reward_rate: RewardRate,
    pub buy_u_switch: bool,
    pub buy_s_switch: bool,

    pub s_price_switch: bool,
    pub num_limit: u8, 
    pub ubox_sale_num_limit: u32,
    pub ubox_sale_num: u32,
    pub ubox_sale_amount: u128,
    pub sbox_sale_num: u32,
    pub sbox_sale_amount: u128,
    pub total_balance_shipwallet: U128,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            magicbox: self.magicbox.clone(),
            usn: self.usn.clone(),
            token_tia: self.token_tia.clone(),
            bank: self.bank.clone(),
            bank_u: self.bank_u.clone(),
            oracle: self.oracle.clone(),
            risker_pool: self.risker_pool.clone(),
            rank_pool: self.rank_pool.clone(),
            ship_pool: self.ship_pool.clone(),
            luck: self.luck.clone(),

            reward_rate: self.reward_rate,
            buy_u_switch: self.buy_u_switch,
            buy_s_switch: self.buy_s_switch,
            s_price_switch: self.s_price_switch,
            num_limit: self.num_limit,
            ubox_sale_num_limit: self.ubox_sale_num_limit,
            ubox_sale_num: self.ubox_sale_num,
            ubox_sale_amount: self.ubox_sale_amount,
            sbox_sale_num: self.sbox_sale_num,
            sbox_sale_amount: self.sbox_sale_amount,
            total_balance_shipwallet: U128(self.total_balance_shipwallet),
        }
    }

    // balanceOf
    // BML-00-05
    pub fn ship_wallet_balance_of( &self, from: AccountId ) -> u128 {
        return self.balances.get(&from).unwrap_or(0);
    }
    // BML-00-04 
    pub fn ship_wallet_total_balance( &self ) -> u128 {
        self.total_balance_shipwallet
    }
    // GetSBoxPrice
    // BML-00-15
    pub fn get_sbox_price(&self) -> u128{
        self.sbox_price_info.sbox_price
    }

    // get price by Oracle. getTWapPrice
    // BML-00-18
    pub fn internal_get_tia_twap_price( &self ) -> u128 {
        // to do 
        //return IOracle(oracle).consult(ssp, 1e18);
        1
    }
   
    // getLatestUBoxSalePool
    pub fn get_latest_u_box_sale_pool(&self) -> UBoxSale {
        return self.ubox_sale_pool.get(self.ubox_sale_pool.len()-1).unwrap().clone();
    }
    
    // getBoxSSPPrice
    pub fn get_box_ssp_price( &self ) -> u128 {
        // ssp's decimals is 18
        let mut twapPrice: u128 = 0;
        
        if self.s_price_switch || self.sbox_price_info.ssp_twap_price_min < self.internal_get_tia_twap_price(){
            twapPrice = self.sbox_price_info.ssp_twap_price_min;
        }
        else{
            twapPrice = self.internal_get_tia_twap_price();
        }

        self.sbox_price_info.sbox_price*(YOCTO18/twapPrice)
    }
    // get sprice swith
    pub fn get_s_price_switch(&self) -> bool {
        self.s_price_switch
    }

    // get ubox switch
    pub fn get_ubox_switch(&self) -> bool {
        self.buy_u_switch
    }

    // get sbox switch
    pub fn get_sbox_switch(&self) -> bool {
        self.buy_s_switch
    }
}
