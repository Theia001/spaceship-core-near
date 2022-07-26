use crate::*;


#[near_bindgen]
impl Contract {
    // invite related
    // checkBind
    pub fn check_bind(&self, from: AccountId ) -> bool {
      let info = self.user_relation.get(&from).unwrap();
      return info.parent != "".to_string();
  }

  // getInfo
  pub fn get_info(&self, from: AccountId) -> Relation {
      let info = self.user_relation.get(&from).unwrap_or(
          Relation{
            parent: "".to_string(),  
            donate: 0, 
            donate_u: 0, 
            num: 0, 
            children: vec![]
         }
      );
      return info;
  }

  /* ==========invite CORE FUNCTION ========== */
  pub fn bind(&mut self, parent: AccountId) {
      //require!(parent != address(0), "Invite: invalid parent address");
      let predecessor_account_id = env::predecessor_account_id();
      require!(parent != predecessor_account_id, "Invite: do not bind yourself");

      let mut parent_relation = self.user_relation.get(&parent).unwrap_or(
         Relation{
            parent: "".to_string(), 
            donate: 0, 
            donate_u: 0, 
            num: 0, 
            children: vec![]
         }
      );
      require!(parent_relation.parent != env::current_account_id().to_string(), "Invite: grandPa must not be yourself");

      let mut relation = self.user_relation.get(&predecessor_account_id).unwrap_or(
         Relation{
            parent: "".to_string(),
            donate: 0, 
            donate_u: 0, 
            num: 0, 
            children: vec![]
         }
      );

      // check if relation.parent is empty
      require!(relation.parent == "".to_string(), "Invite: already bind");

      // check if relation.parent is empty
      relation.parent =  parent.clone().to_string();

      self.user_relation.insert(&predecessor_account_id,&relation);
      
      // update parent_relation data
      parent_relation.num = parent_relation.num + 1;
      parent_relation.children.push(Children{
          addr:predecessor_account_id.clone(),
          create_time: nano_to_sec(env::block_timestamp()),
          });
      self.user_relation.insert(&parent, &parent_relation);
      // emit event
      Event::Bind{from:&predecessor_account_id, parent: &parent}.emit();
  }
}
impl Contract {
   // invite related
   pub fn reward_token(&mut self, from: AccountId, to: AccountId, amount: u128) {
      
      //IShipWallet(shipWallet).add(to, amount);
      self.internal_ship_wallet_add(to.clone(), amount);

      let mut relation = self.user_relation.get(&to.clone()).unwrap();
      relation.donate += amount;
      self.user_relation.insert(&to,&relation);
      
      // emit event
      Event::RewardToken{from:&from, to: &to, amount: &U128(amount)}.emit();
  }

  pub fn reward_u(&mut self, from: AccountId, to: AccountId, amount: u128) {
      ext_fungible_token::ft_transfer(
          to.clone(),
          U128(amount),
          None,
          self.usn.clone(),
          1,
          GAS_FOR_BATCH_TRANSFER
      );
      
      let mut relation = self.user_relation.get(&to).unwrap();
      relation.donate_u += amount;
      self.user_relation.insert(&to,&relation);
      // emit event
      Event::RewardU{from:&from, to: &to, amount: &U128(amount)}.emit();
  }   
}