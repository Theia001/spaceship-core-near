use crate::*;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub box_id: AccountId,
    pub shippool_id: AccountId,
    pub shipmarket_id: AccountId,
    pub auction_id: AccountId,
    pub luckpool_id: AccountId,
    pub next_id: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Debug, Clone))]
pub struct SpaceShipSupply {
    pub supply: U128,
    pub burned: U128,
    pub owners: U128,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            box_id: self.box_id.clone(),
            shippool_id: self.shippool_id.clone(),
            shipmarket_id: self.shipmarket_id.clone(),
            auction_id: self.auction_id.clone(),
            luckpool_id: self.luckpool_id.clone(),
            next_id: self.next_id,
        }
    }

    pub fn get_ship_icon(&self, ship_type_sub_type: String) -> Option<String> {
        self.icons.get(&ship_type_sub_type)
    }
}
