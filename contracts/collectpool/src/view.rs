use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub is_init_slot: bool,
    pub token: AccountId,
    pub spaceship: AccountId,
    pub boxmall: AccountId,
    pub start_block: u64,
    pub end_block: u64,
    pub last_reward_block: u64,
    pub total_capacity: u32,
    pub per_block_reward: u128,
    pub total_period: u32, 
    pub decay_period: u64,
    pub decay_rate: u64,
    pub reward_rate: u64,
    pub format_rate: u64,
    pub reward_per_token_stored: u128,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            is_init_slot:  self.is_init_slot, 
            token:  self.token_tia.clone(),
            spaceship:  self.spaceship.clone(),
            boxmall:  self.boxmall.clone(),
       
            /* ========== VARIABLE SETTING ========== */
            start_block: self.start_block,
            end_block: self.end_block,
            last_reward_block: self.last_reward_block,
            total_capacity: self.total_capacity,
            per_block_reward: self.per_block_reward,
            total_period: self.total_period,
            decay_period: self.decay_period,
            decay_rate: self.decay_rate,
            reward_rate: self.reward_rate,
            format_rate: self.format_rate,
       
            reward_per_token_stored: self.reward_per_token_stored,
        }
    }

    /* ========== VIEW FUNCTION ========== */
    pub fn get_slot_list( &self, from: AccountId) -> Vec<SlotList> {
        let mut list: Vec<SlotList> = vec![];
        log!("collectpool.get_slot_list. self.slot.len = {:#?}", self.slot.len());
        for i in 0..self.slot.len(){
            let stake_slot_vec: Vec<StakeSlot> = self.slot_info.get(&from).expect("No Stake Slot");
            let mut slot_list_node: SlotList = SlotList{
                slot_index: 0,
                price: 0,
                enable: false,
                token_id: "".to_string(),
                capacity: 0,
                ship_type: 0,
                ship_subtype: 0,
            };
            let slot_node = self.slot.get(i).expect("No Slot");
            slot_list_node.slot_index = i as u32;
            slot_list_node.price = slot_node.price;
            slot_list_node.enable = slot_node.enable || stake_slot_vec[i as usize].enable;
            let token_id: TokenId = stake_slot_vec[i as usize].token_id.clone();
            slot_list_node.token_id = token_id.clone();
            if token_id != "".to_string(){
                let ship_type = self.internal_get_ship_type_by_token_id(token_id.clone());
                let ship_subtype = self.internal_get_ship_subtype_by_token_id(token_id.clone());
                let ship_capacity = self.internal_get_ship_capacity_by_token_id(token_id.clone());
                slot_list_node.capacity = ship_capacity as u32;
                slot_list_node.ship_type = ship_type;
                slot_list_node.ship_subtype = ship_subtype;
            }
            log!("collectpool.get_slot_list. slot_list_node = {:#?}", slot_list_node.clone());
            list.push(slot_list_node);
        }
        return list;
    }

    pub fn get_extra_capacity(&self, from: AccountId, slot_index: u64) -> u32 {
        let slot_temp: Slot = self.slot.get(slot_index).expect("Invalid slot_index");
        let mut num: u8 = 0;
        let mut extra: u32 = 0;

        for i in 0..self.slot.len() {
            if let Some(slot) = self.slot.get(i) {
                if slot_temp.ship_type == slot.ship_type {
                    let stake_slot_vec: Vec<StakeSlot> = self.slot_info.get(&from).expect("No_StakeSlot");
                    let stake_slot: StakeSlot = stake_slot_vec[i as usize].clone();

                    if stake_slot.token_id != "".to_string(){
                        num += 1;
                    }
                }                
            }
        }

        if slot_temp.ship_type == TYPE_SHIP_D && num == 32 {
            extra = 5;
        }

        if slot_temp.ship_type == TYPE_SHIP_C && num == 16 {
            extra = 10;
        }

        if slot_temp.ship_type == TYPE_SHIP_B && num == 8 {
            extra = 15;
        }

        if slot_temp.ship_type == TYPE_SHIP_A && num == 4 {
            extra = 20;
        }
        extra
    }
}