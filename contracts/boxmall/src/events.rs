use near_sdk::{
    AccountId, log,
    serde::{Serialize},
    serde_json::{json},
    json_types::U128,
};

const EVENT_STANDARD: &str = "boxmall";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    BuyU {
        caller_id: &'a AccountId,
        buyer_id: &'a AccountId,
        amount: &'a U128,
        num: u32,
    },
    BuyS {
        caller_id: &'a AccountId,
        buyer_id: &'a AccountId,
        amount: &'a U128,
        num: u32,
    },
    SetSwitch {
        caller_id: &'a AccountId,
        u_switch: bool,
        s_switch: bool,
    },
    SetSBoxPrice {
        caller_id: &'a AccountId,
        sbox_price: &'a U128,
    },
    SetSSPTWapPriceMin {
        caller_id: &'a AccountId,
        set_tia_twap_price_min: &'a U128,
    },
    SetRewardRate {
        caller_id: &'a AccountId,
        old_ship_reward_rate: u8, new_ship_reward_rate: u8,
        old_risker_reward_rate: u8, new_risker_reward_rate: u8,
        old_bank_reward_rate: u8, new_bank_reward_rate: u8
    },
    SetUBoxSaleNumLimit {
        caller_id: &'a AccountId,
        limit: u32,
    },
    SetNumLimit {
        caller_id: &'a AccountId,
        limit: u8,
    },

    Bind{
        from: &'a AccountId,
        parent: &'a AccountId,
    },
    RewardToken{
        from: &'a AccountId,
        to: &'a AccountId,
        amount: &'a U128,
    },
    RewardU{
        from: &'a AccountId,
        to: &'a AccountId,
        amount: &'a U128,
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

    fn bob() -> AccountId {
        AccountId::new_unchecked("bob".to_string())
    }

    #[test]
    fn buy_u() {

    }

}