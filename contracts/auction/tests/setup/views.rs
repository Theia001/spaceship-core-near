use crate::setup::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub token_id: AccountId,
    pub spaceship_id: AccountId,
    pub team_id: AccountId,
    pub rebate_rate: u8, 
    pub duration_sec: u64,
}

impl Env {

    pub fn get_metadata(&self) -> Metadata {
        self.owner
        .view_method_call(
            self.auction.contract.get_metadata()
        ).unwrap_json::<Metadata>()
    }

    pub fn get_auction_list(&self) -> Vec<AuctionInfo> {
        self.owner
            .view_method_call(
                self.auction.contract.get_auction_list(None, None)
            ).unwrap_json::<Vec<AuctionInfo>>()
    }

    pub fn get_auction_count(&self) -> u64 {
        self.owner
            .view_method_call(
                self.auction.contract.get_auction_count()
            ).unwrap_json::<u64>()
    }

    pub fn get_current_timestamp(&self) -> u64 {
        self.owner
            .view_method_call(
                self.auction.contract.get_current_timestamp()
            ).unwrap_json::<u64>()
    }

    pub fn get_auction_by_id(&self, auction_id: AuctionId) -> Option<AuctionInfo> {
        self.owner
            .view_method_call(
                self.auction.contract.get_auction_by_id(auction_id)
            ).unwrap_json::<Option<AuctionInfo>>()
    }
}