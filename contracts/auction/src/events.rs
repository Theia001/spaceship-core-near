use near_sdk::{
    AccountId, log,
    serde::{Serialize},
    serde_json::{json},
    json_types::U128,
};

const EVENT_STANDARD: &str = "auction";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    Buy {
        buyer_id: &'a AccountId,
        auction_id: u64,
        token_id: &'a String,
        price: &'a U128,
    },

    UpdateAuctionInfo {
        caller_id: &'a AccountId,
        auction_id: u64,
        price: &'a U128,
        up_price: &'a U128,
        start: u64,
        end: u64,
    },  

}

impl Event<'_> {
    pub fn emit(&self) {
        emit_event(&self);
    }
}

// Emit event that follows NEP-297 standard: https://nomicon.io/Standards/EventsFormat
// Arguments
// * `standard`: name of standard, e.g. nep171
// * `version`: e.g. 1.0.0
// * `event`: type of the event, e.g. nft_mint
// * `data`: associate event data. Strictly typed for each set {standard, version, event} inside corresponding NEP
pub (crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!({
        "standard": EVENT_STANDARD,
        "version": EVENT_STANDARD_VERSION,
        "event": result["event"],
        "data": [result["data"]]
    })
    .to_string();
    log!(format!("EVENT_JSON:{}", event_json));
}


#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{test_utils, AccountId};

    fn alice() -> AccountId {
        AccountId::new_unchecked("alice".to_string())
    }

    #[test]
    fn event_buy() {
        let buyer_id = &alice();
        let auction_id: u64 = 1;
        let token_id = &"1".to_string();
        let price = &U128(100);


        Event::Buy { buyer_id: &buyer_id, auction_id, token_id, price }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"auction","version":"1.0.0","event":"buy","data":[{"buyer_id":"alice","auction_id":1,"token_id":"1","price":"100"}]}"#
        );
    }

    fn event_update_auction_info() {
        let caller_id = &alice();
        let auction_id: u64 = 1;
        let price = &U128(100);
        let up_price = &U128(10);
        let start: u64 = 1000;
        let end: u64 = 1020;

        Event::UpdateAuctionInfo { caller_id: &caller_id, auction_id, price, up_price, start, end }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"auction","version":"1.0.0","event":"update_auction_info","data":[{"caller_id":"alice","auction_id":1,"price":"100","up_price":"10","start":100, "end": 1010}]}"#
        );
    }

}