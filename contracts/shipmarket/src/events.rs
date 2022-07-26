use near_sdk::{
    AccountId, log,
    serde::{Serialize},
    serde_json::{json},
    json_types::U128,
};

const EVENT_STANDARD: &str = "shipmarket";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    SellEvent {
        order_id: u64,
        previous_owner_id:  &'a AccountId,
        token_id: &'a String,
        order_status: u8,
        amount: U128,
        timestamp: u64,
        ship_type: u8,
        ship_subtype: u8
    },

    BuyEvent {
        order_id: u64, 
        sender_id:  &'a AccountId,
        order_status: u8,
        amount: U128,
        fee: U128, 
        update_time: u64,
    }, 

    CancelEvent{
        order_id: u64,
        seller_id: &'a AccountId,
        order_status: u8,
        update_time: u64,
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
    fn event_sell() {
        let order_id: u64 = 1;
        let previous_owner_id = &alice();
        let token_id = &"1".to_string();
        let order_status: u8 = 1;
        let amount = U128(100);
        let timestamp: u64 = 1234;
        let ship_type: u8 = 1;
        let ship_subtype: u8 = 2;


        Event::SellEvent { order_id, previous_owner_id, token_id, order_status, amount, timestamp, ship_type, ship_subtype }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"shipmarket","version":"1.0.0","event":"sell_event","data":[{"order_id":1,"previous_owner_id":"alice","token_id":"1","order_status":1,"amount":"100","timestamp":1234,"ship_type":1,"ship_subtype":2}]}"#
        );
    }

    #[test]
    fn event_buy() {
        let order_id: u64 = 1;
        let sender_id = &alice();
        let order_status: u8 = 1;
        let amount = U128(100);
        let fee = U128(10);
        let update_time: u64 = 1234;


        Event::BuyEvent { order_id, sender_id, order_status, amount, fee, update_time }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"shipmarket","version":"1.0.0","event":"buy_event","data":[{"order_id":1,"sender_id":"alice","order_status":1,"amount":"100","fee":"10","update_time":1234}]}"#
        );
    }

    #[test]
    fn event_cancel() {
        let order_id: u64 = 1;
        let seller_id = &alice();
        let order_status: u8 = 1;
        let update_time: u64 = 1234;

        Event::CancelEvent { order_id, seller_id, order_status,  update_time }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"shipmarket","version":"1.0.0","event":"cancel_event","data":[{"order_id":1,"seller_id":"alice","order_status":1,"update_time":1234}]}"#
        );
    }
}