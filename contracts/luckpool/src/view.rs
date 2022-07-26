use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub token_tia: AccountId,
    pub spaceship: AccountId,
    pub duration: u64,
    pub id: u64,
    pub reward: U128,
    pub settle_reward: U128,
    pub claimed_reward: U128,
    pub reward_rate1: u8,
    pub reward_rate2: u8,
    pub reward_rate3: u8,
    pub balance: U128,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            token_tia: self.token_tia.clone(),
            spaceship: self.spaceship.clone(),
            duration: self.duration,
            id: self.id,
            reward: U128(self.reward),
            settle_reward: U128(self.settle_reward),
            claimed_reward: U128(self.claimed_reward),
            reward_rate1: self.reward_rate1,
            reward_rate2: self.reward_rate2,
            reward_rate3: self.reward_rate3,
            balance: U128(self.balance),
        }
    }

    pub fn get_round_length(&self) -> u64 {
        self.round_history.len()
    }

    pub fn get_latest_index(&self) -> u64 {
        self.round_history.len() - 1
    }

    pub fn get_round_info(&self, index: u64) -> Round {
        if index < self.round_history.len() {
            self.round_history.get(index).unwrap()
        } else {
            Round {
                index: 0,
                settle_reward: 0,
                total_amount: 0,
                end: 0,
                winner1: AccountId::new_unchecked("00".to_string()),
                winner2: AccountId::new_unchecked("00".to_string()),
                winner3: AccountId::new_unchecked("00".to_string()),
                reward1: 0,
                reward2: 0,
                reward3: 0,
                settle: false,
            }
        }
    }

    pub fn get_round_history(&self) -> Vec<Round> {
        let length = self.round_history.len();
        let begin = 0;

        let mut list = Vec::with_capacity(length as usize);
        for i in begin..length {
            let round = self.round_history.get(i).unwrap();
            list.push(round);
        }
        list
    }

    pub fn get_user_info(&self, account_id: AccountId) -> User {
        let mut count = 0;
        let mut total_count = 0;

        if let Some(tokens_per_owner) = &self.tokens.tokens_per_owner {
            let set = tokens_per_owner.get(&account_id).unwrap_or(UnorderedSet::new(StorageKey::TokensPerOwner {
                account_hash: env::sha256(account_id.as_bytes()),
            }));

            let current_round = self.round_history.len() - 1;

            set.iter().for_each(|item| {
                if item.starts_with(format!("{}:", current_round).as_str()) {
                    count += 1;
                }
            });

            let keys = self.tokens.owner_by_id.keys();
            keys.for_each(|token_id| {
                if token_id.starts_with(format!("{}:", current_round).as_str()) {
                    total_count += 1;
                }
            });
        }

        User {
            amount: count,
            total_amount: total_count,
        }
    }
}