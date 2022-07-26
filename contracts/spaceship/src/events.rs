use near_sdk::{log, serde::Serialize, serde_json::json, AccountId};

const EVENT_STANDARD: &str = "spaceship";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    UpgradeEvent {
        sender_id: &'a AccountId,
        token_id_1: &'a String,
        token_id_2: &'a String,
        token_id: &'a String,
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
pub(crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
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
    fn event_upgrade_event() {
        let buyer_id = &alice();
        let token_id_1 = &"1".to_string();
        let token_id_2 = &"2".to_string();
        let token_id = &"3".to_string();

        Event::UpgradeEvent {
            sender_id: &buyer_id,
            token_id_1,
            token_id_2,
            token_id,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"spaceship","version":"1.0.0","event":"upgrade_event","data":[{"sender_id":"alice","token_id_1":"1","token_id_2":"2","token_id":"3"}]}"#
        );
    }
}
