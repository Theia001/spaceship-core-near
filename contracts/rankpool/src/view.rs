use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub token_tia: AccountId,
    pub boxmall: AccountId,
    pub shipmarket: AccountId,
    pub duration_sec: TimeStampSec,
    pub balance: u128,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            token_tia: self.token_tia.clone(),
            boxmall: self.boxmall.clone(),
            shipmarket: self.shipmarket.clone(),
            duration_sec: self.duration_sec,
            balance: self.balance,
        }
    }
}