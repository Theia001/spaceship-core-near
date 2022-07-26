use crate::*;

impl Contract {
    pub fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "ERR_NOT_ALLOWED"
        );
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn set_owner(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.owner_id = owner_id;
    }

    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let contract: Contract = env::state_read().expect("ERR_NOT_INIT");
        contract
    }

    pub fn stake(&mut self, index: u64, amount: U128) {
        let mut round = self.round_history.get(index ).unwrap();
        require!(amount.0 > 0, "Amount invalid");
        require!(round.end > nano_to_sec(env::block_timestamp()), "This round is end");
        require!(!round.settle, "This round is settle");

        self.id += 1;
        let order_id: u64 = self.id;

        let predecessor_account_id = env::predecessor_account_id();

        round.total_amount += amount.0;

        // 调用spaceship burn
        ext_spaceship::burn_eng_for_user(
            predecessor_account_id.clone(),
            amount,
            self.spaceship.clone(),
            1,
            GAS_FOR_TRANSFER);

        let round = self.round_history.len() + 1;
        let token_id: TokenId = format!("{}", round); // 这里只是轮次，还需要拼接mint后的
        let token_metadata: Option<TokenMetadata> = Some(TokenMetadata {
            title: Some("NFT TICKET".to_string()),
            description: Some("NFT TICKET".to_string()),
            media: None,
            media_hash: None,
            copies: None,
            issued_at: Some(nano_to_sec(env::block_timestamp()).to_string()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        });
        self.tokens.internal_mint(token_id, predecessor_account_id.clone(), token_metadata, amount.0);

        Event::Stake{sender:&predecessor_account_id, order_id, index, amount}.emit();
    }

    #[private]
    pub fn draw(&mut self, index: u64, reward: u128, rate: u8) -> (AccountId, u128) {
        let winner = self.tokens.draw(index as u128);

        if winner == AccountId::new_unchecked("00".to_string()) {
            return (AccountId::new_unchecked("00".to_string()), 0);
        }

        // TODO ITicket(allTicket[_index]).burnFrom(winner, balance);
        let reward = reward * rate as u128 / RATE_DENOMINATOR as u128;

        let mut receiver_ids: Vec<String> = vec![];
        let mut amounts: Vec<U128> = vec![];

        receiver_ids.push(winner.clone().to_string());
        amounts.push(U128(reward));

        self.claimed_reward = self.claimed_reward + reward;
        (winner, reward)
    }

    pub fn settle(&mut self) {
        let index = self.get_latest_index();
        let mut round = self.round_history.get(index).unwrap();
        require!(!round.settle, "This round is settle");

        // 本次应发奖励
        self.reward = self.reward + self.claimed_reward - self.settle_reward;
        let reward: u128 = self.reward;
        let (winner1, reward1) = self.draw(index, reward, self.reward_rate1);
        let (winner2, reward2) = self.draw(index, reward, self.reward_rate2);
        let (winner3, reward3) = self.draw(index, reward, self.reward_rate3);

        round.winner1 = winner1;
        round.winner2 = winner2;
        round.winner3 = winner3;

        round.reward1 = reward1;
        round.reward2 = reward2;
        round.reward3 = reward3;

        round.settle_reward = reward1 + reward2 + reward3;
        round.settle = true;
        self.round_history.replace(index, &round);
        self.settle_reward = self.settle_reward + round.settle_reward;
        self.create_round();
    }
}