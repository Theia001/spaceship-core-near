use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub ship_contract_id: AccountId,
    pub token_tia: AccountId,
    pub total_reward: u128,
    pub total_claimed_reward: u128,
    pub pioneer_max: u32,
    pub balance: Balance,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            ship_contract_id: self.ship_contract_id.clone(),
            token_tia: self.token_tia.clone(),
            total_reward: self.total_reward,
            total_claimed_reward: self.total_claimed_reward,
            pioneer_max: self.pioneer_max,
            balance: self.balance,
        }
    }

    /* ========== VIEW FUNCTION ========== */
    // SPL-00-10
    pub fn get_current_epoch(& self ) ->RoundVO {
        let mut vo: RoundVO = RoundVO{
            pioneer: vec![],
            round_reward: U128(0),
            index: 0,
        };

        if self.round_history.len() == 0 {
            return vo;
        }

        let round: Round = self.round_history.get(self.round_history.len() - 1).unwrap().clone();
        let mut pioneer: Vec<AccountId> = vec![];

        for i in 0..round.pioneer.len(){
            pioneer.push(round.pioneer[i].clone());
        }

        let reward: u128 = self.balance + self.total_claimed_reward - self.total_reward;

        vo.pioneer = pioneer;
        vo.round_reward = U128(reward);
        vo.index = (self.round_history.len() - 1) as u32;
        return vo;
    }

    // SPL-00-11
    pub fn get_user_info(&mut self, to: AccountId ) -> UserInfo {
        let mut u: UserInfo = self.user_info.get(&to).unwrap_or(
            UserInfo{
                total_burned: 0,
                burned: 0,
                pioneer_token_amount: U128(0),
                need_init: false,
                a: vec![],
                b: vec![],
                c: vec![],
                d: vec![],
            }
        ).clone();

        if u.need_init == true{
            // only keep u.total_burned
            u.burned = 0;
            u.pioneer_token_amount = U128(0);
            u.need_init = false;
            u.a = vec![0;TYPE_A_MAX.into()];
            u.b = vec![0;TYPE_B_MAX.into()];
            u.c = vec![0;TYPE_C_MAX.into()];
            u.d = vec![0;TYPE_D_MAX.into()];

            self.user_info.insert(&to,&u);
        }
        return u;
    }

    // history round does't include current round.
    // SPL-00-09
    pub fn get_epoch_history_list(&self, page: u64,  size: u64) -> ( u64, Vec<RoundHistoryVO>) {
        require!(page >= 1, "invalid param");
        let max: u64;
        let mut list: Vec<RoundHistoryVO> = vec![];

        if self.round_history.len() == 0 {
            return (0, list);
        }

        if page*size >= self.round_history.len() - 1 {
            max = self.round_history.len() - 1;
        } else {
            max = page*size;
        }

        if max == 0 {
            return (0, list);
        }
        let begin: u64 = (page - 1)* size;

        for i in begin..max{
            let round_index: u64 = self.round_history.len() - i - 2;
            let round: Round = self.round_history.get(round_index).unwrap().clone();

            let mut round_history_node: RoundHistoryVO = RoundHistoryVO{
                index: round_index as u32,
                round_reward: U128(round.round_reward),
                reward_time: round.reward_time,
                avg_reward: round.avg_reward,
                pioneer: vec![],
            };
            for j in 0..round.pioneer.len(){
                let hist_key: RoundIdAcc = format!("{}:{}", j, round.pioneer[j]);
                let pioneer:Pioneer = self.pioneer_history.get(&hist_key).unwrap().clone();
                round_history_node.pioneer.push(pioneer);
            }
            list.push(round_history_node);
          
        }

        let total = self.round_history.len() - 1;
        (total, list)
    }
}