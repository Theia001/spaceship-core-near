use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub token_id: AccountId,
    pub spaceship_id: AccountId,
    pub team_id: AccountId,
    pub rebate_rate: u8, 
    pub duration_sec: u64,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            token_id: self.token_id.clone(),
            spaceship_id: self.spaceship_id.clone(),
            team_id: self.team_id.clone(),
            rebate_rate: self.rebate_rate, 
            duration_sec: self.duration_sec,
        }
    }

    /* ========== VIEW FUNCTION ========== */
    // AUC_O0_09
    pub fn get_auction_list(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<AuctionInfo> {
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.pool.len());
        (from_index..std::cmp::min(from_index + limit, self.pool.len()))
            .map(|index| self.pool.get(index).unwrap())
            .collect()
    }

    // AUC_O0_11
    pub fn get_auction_count(&self) -> u64 {
        self.pool.len()
    }

    // return timestamp in seconds
    pub fn get_current_timestamp(&self) -> u64 {
        return nano_to_sec(env::block_timestamp());
    }

    // AUC_O0_10
    pub fn get_auction_by_id(&self, auction_id: AuctionId) -> Option<AuctionInfo> {
        return self.pool.get(auction_id);
    }
}

/* ========== INTERNAL FUNCTION ========== */
impl Contract{

}