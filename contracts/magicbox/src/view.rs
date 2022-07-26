use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub spaceship_contract_id: AccountId,
    pub next_token_idx: u64,
    pub num_limit: u8, 
    pub burn_balance: u64,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            spaceship_contract_id: self.spaceship_contract_id.clone(),
            next_token_idx: self.next_token_idx,
            num_limit: self.num_limit,
            burn_balance: self.burn_balance,
        }
    }
}