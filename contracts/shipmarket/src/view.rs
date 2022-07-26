use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub spaceship: AccountId,
    pub token_tia: AccountId,
    pub ship_pool: AccountId,
    pub bank: AccountId,
    pub risker_pool: AccountId,
    pub rank_pool: AccountId,
    pub luck_pool: AccountId,

    pub ship_reward_rate: u8,
    pub risker_reward_rate: u8,
    pub bank_reward_rate: u8,
    pub luck_reward_rate: u8,
    pub rank_reward_rate: u8,
    pub fee_rate: u8,
    pub next_order_id: u64,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            spaceship: self.spaceship.clone(),
            token_tia: self.token_tia.clone(),
            ship_pool: self.ship_pool.clone(),
            bank: self.bank.clone(),
            risker_pool: self.risker_pool.clone(),
            rank_pool: self.rank_pool.clone(),
            luck_pool: self.luck_pool.clone(),

            ship_reward_rate: self.ship_reward_rate,
            risker_reward_rate: self.risker_reward_rate,
            bank_reward_rate: self.bank_reward_rate,
            luck_reward_rate: self.luck_reward_rate,
            rank_reward_rate: self.rank_reward_rate,
            fee_rate: self.fee_rate,
            next_order_id: self.next_order_id,
        }
    }

    pub fn get_target_fee(&self, ship_type: u8) -> Vec<U128> {
        let temp = self.target_mint_fee.get(&ship_type).unwrap();
        let mut fees: Vec<U128> = vec![];
        
        for x in &temp {
            fees.push(U128(x.clone()));
        }
        fees
    }

    pub fn get_no_target_fee(&self, ship_type: u8)  -> Vec<U128>{
        let temp = self.target_mint_fee.get(&ship_type).unwrap();
        let mut fees: Vec<U128> = vec![];
        
        for x in &temp {
            fees.push(U128(x.clone()));
        }
        fees
    }

    pub fn get_buy_fee_rate( &mut self ) -> u8 {
        self.fee_rate
    }

    // return order_id, Order
    pub fn list_orders(&self, from_index: Option<u64>, limit: Option<u64>) -> HashMap<u64, Order> {
        let keys = self.order_map.keys_as_vector();
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.order_map.len());

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| {
                (
                    keys.get(index).unwrap(),
                    self.order_map.get(&keys.get(index).unwrap()).unwrap()
                )
            })
            .collect()
    }

    // return buy orders for user,
    pub fn list_buy_orders(&self, account_id: AccountId, from_index: Option<u64>, limit: Option<u64>) -> Vec<Order> {
        let order_ids: Vec<u64> = self.buy_order_map.get(&account_id.clone()).unwrap_or(vec![]);
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(order_ids.len() as u64);

        (from_index..std::cmp::min(from_index + limit, order_ids.len() as u64))
        .map(|index| 
            (self.order_map.get(&(order_ids[index as usize] as u64)).unwrap())
        )
        .collect()
    }

    // return all orders for user
    pub fn list_user_orders(&self, account_id: AccountId, from_index: Option<u64>, limit: Option<u64>)  -> Vec<Order> {
        let keys = self.order_map.keys_as_vector();
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.order_map.len());

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .filter(|index| 
                (self.order_map.get(&keys.get(*index).unwrap()).unwrap().seller == account_id )
            )
            .map(|index| 
                (self.order_map.get(&keys.get(index).unwrap()).unwrap())
            )
            .collect()
    }
}
