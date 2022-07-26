use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub token_tia: AccountId,
    pub boxmall: AccountId,
    pub reward_rate: u8,
    pub duration_sec: u64,
    pub shot_duration_sec: u64,
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
            reward_rate: self.reward_rate,
            duration_sec: self.duration_sec,
            shot_duration_sec: self.shot_duration_sec,
        }
    }
}