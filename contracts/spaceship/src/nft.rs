use crate::*;
use near_contract_standards::non_fungible_token::metadata::{
    NonFungibleTokenMetadataProvider, TokenMetadata,
};

pub const TYPE_S: u8 = 5;
pub const TYPE_S_COUNT: u64 = 4;
pub const TYPE_A: u8 = 4;
pub const TYPE_B: u8 = 3;
pub const TYPE_C: u8 = 2;
pub const TYPE_D: u8 = 1;

pub const TYPE_A_MAX: u8 = 4;
pub const TYPE_B_MAX: u8 = 8;
pub const TYPE_C_MAX: u8 = 16;
pub const TYPE_D_MAX: u8 = 32;

pub fn parse_token_id(token_id: &TokenId) -> ShipElements {
    let items: Vec<&str> = token_id.split(":").collect();
    ShipElements {
        prefix_id: items
            .get(0)
            .expect("ILLIGAL_TOKEN_ID")
            .parse::<u64>()
            .expect("ILLIGAL_TOKEN_ID"),
        ship_type: items
            .get(1)
            .expect("ILLIGAL_TOKEN_ID")
            .parse::<u8>()
            .expect("ILLIGAL_TOKEN_ID"),
        ship_sub_type: items
            .get(2)
            .expect("ILLIGAL_TOKEN_ID")
            .parse::<u8>()
            .expect("ILLIGAL_TOKEN_ID"),
        capacity: items
            .get(3)
            .expect("ILLIGAL_TOKEN_ID")
            .parse::<u32>()
            .expect("ILLIGAL_TOKEN_ID"),
    }
}

pub fn gen_token_id(element: &ShipElements) -> TokenId {
    format!(
        "{}:{}:{}:{}",
        element.prefix_id, element.ship_type, element.ship_sub_type, element.capacity
    )
}

/// custom view interfaces
#[near_bindgen]
impl Contract {
    /// [SSP-00-21]
    pub fn get_spaceship_supply(&self) -> SpaceShipSupply {
        SpaceShipSupply {
            supply: self.tokens.nft_total_supply().into(),
            burned: self.tokens.internal_burned_count().into(),
            owners: self.tokens.internal_user_count().into(),
        }
    }

    /// [SSP-00-20]
    /// Return avtive ship's ShipElements in list for given owner
    pub fn get_spaceship_list_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<ShipElements> {
        self.tokens
            .nft_tokens_for_owner(
                account_id,
                from_index.and_then(|x| Some(U128(x as u128))),
                limit,
            )
            .into_iter()
            .map(|token| parse_token_id(&token.token_id))
            .collect()
    }

    // SSP-00-07 total burned token count by type_detail
    pub fn total_burnt_subtype_of(&self, ship_type: u8, ship_sub_type: u8) -> u32 {
        let ship_type_detail: String = format!("{}:{}", ship_type, ship_sub_type);
        self.total_supplies
            .get(&ship_type_detail)
            .unwrap_or(ShipSupply {
                active: 0,
                burned: 0,
            })
            .burned
    }

    // SSP-00-09 total active token count by type_detail
    pub fn total_supply_subtype_of(&self, ship_type: u8, ship_sub_type: u8) -> u32 {
        let ship_type_sub_type: String = format!("{}:{}", ship_type, ship_sub_type);
        self.total_supplies
            .get(&ship_type_sub_type)
            .unwrap_or(ShipSupply {
                active: 0,
                burned: 0,
            })
            .active
    }

    // SSP-00-03 duplicate interface with NEP171
    pub fn balance_of(&self, account_id: AccountId) -> u32 {
        self.tokens.nft_supply_for_owner(account_id).0 as u32
    }

    // SSP-00-04 get active ship count for given owner and given ship type
    pub fn balance_type_of(&self, account_id: AccountId, ship_type: u8) -> u32 {
        let owner_supplies = self.supply_per_owner.get(&account_id).unwrap_or_else(|| {
            UnorderedMap::new(StorageKey::SupplyPerOwner {
                account_hash: env::sha256(account_id.as_bytes()),
            })
        });

        let key = format!("{}", ship_type);
        owner_supplies
            .get(&key)
            .unwrap_or(ShipSupply {
                active: 0,
                burned: 0,
            })
            .active
    }

    // SSP-00-05 get active ship count for given owner and given ship type and its sub type
    pub fn balance_subtype_of(
        &self,
        account_id: AccountId,
        ship_type: u8,
        ship_sub_type: u8,
    ) -> u32 {
        let owner_supplies = self.supply_per_owner.get(&account_id).unwrap_or_else(|| {
            UnorderedMap::new(StorageKey::SupplyPerOwner {
                account_hash: env::sha256(account_id.as_bytes()),
            })
        });

        let key = format!("{}:{}", ship_type, ship_sub_type);
        owner_supplies
            .get(&key)
            .unwrap_or(ShipSupply {
                active: 0,
                burned: 0,
            })
            .active
    }

    // SSP-00-06
    pub fn owner_of(&self, token_id: TokenId) -> AccountId {
        self.tokens.owner_by_id.get(&token_id).unwrap()
    }
}

/// custom changable interfaces
#[near_bindgen]
impl Contract {
    // [SSP-00-01]
    pub fn batch_mint(
        &mut self,
        owner_id: AccountId,
        ship_types: Vec<String>,
        ship_sub_types: Vec<String>,
    ) {
        require!(
            env::predecessor_account_id() == self.box_id,
            "ERR_NOT_ALLOWED"
        );

        let mut token_ids: Vec<TokenId> = vec![];

        for tup in ship_types.iter().zip(ship_sub_types.iter()) {
            let (ship_type, ship_sub_type) = tup;
            let ship_type = ship_type.parse::<u8>().unwrap();
            let ship_sub_type = ship_sub_type.parse::<u8>().unwrap();
            require!(
                ship_type >= TYPE_D && ship_type <= TYPE_A,
                "ERR_ILLEGAL_SHIP_TYPE"
            );
            let capacity: u32 = self.internal_random_spaceship_capacity(&ship_type);
            let token_id = gen_token_id(&ShipElements {
                prefix_id: self.next_id,
                ship_type,
                ship_sub_type,
                capacity,
            });
            require!(
                self.tokens.owner_by_id.get(&token_id).is_none(),
                "ERR_TOKEN_ID_CONFLICT"
            );
            self.next_id += 1;

            self.mint_ship_with_supply_updated(owner_id.clone(), &token_id);

            token_ids.push(token_id);

            // todo: emit EVENT
        }

        if token_ids.len() > 0 {
            // register ship, no need to attach callback
            ext_shippool::batch_register_ships(
                token_ids.clone(),
                owner_id.clone(),
                self.shippool_id.clone(),
                0,
                GAS_FOR_REGISTER_SHIP,
            );
        }
    }

    /// [SSP-00-15] only for distribute S-class spaceship by auction contract
    pub fn nft_payout(&mut self, receiver_id: AccountId, token_id: TokenId) {
        let predecessor_id = env::predecessor_account_id();
        require!(predecessor_id == self.auction_id, "Invalid contract id");
        require!(
            parse_token_id(&token_id).ship_type == TYPE_S,
            "ERR_ILLIGAL_SHIP_TYPE"
        );

        self.tokens.internal_transfer(
            &env::current_account_id(),
            &receiver_id,
            &token_id,
            None,
            None,
        );
        self.update_owner_supply(
            &token_id,
            Some(env::current_account_id()),
            Some(receiver_id.clone()),
        );
    }

    // [SSP-00-17] user initiative to burn;
    #[payable]
    pub fn user_burn(&mut self, token_id: TokenId) {
        assert_one_yocto();
        let predecessor_id = env::predecessor_account_id();
        require!(
            predecessor_id == self.tokens.owner_by_id.get(&token_id).unwrap(),
            "ERR_NOT_NFT_OWNER"
        );
        self.burn_ship_with_supply_updated(
            &predecessor_id,
            &token_id,
            Some(predecessor_id.clone()),
        );
    }

    // [SSP-00-14]
    #[payable]
    pub fn upgrade_spaceship(
        &mut self,
        owner_id: AccountId,
        token_id_1: TokenId,
        token_id_2: TokenId,
        target_sub_type: u8,
        eng_amount: U128,
    ) {
        assert_one_yocto();
        let predecessor_id = env::predecessor_account_id();
        require!(predecessor_id == self.shipmarket_id, "ERR_NOT_SHIPMARKET");

        let ship1_type = parse_token_id(&token_id_1).ship_type;
        let ship2_type = parse_token_id(&token_id_2).ship_type;

        require!(
            ship1_type == ship2_type,
            "ShipFactory: material is must same type"
        );
        require!(
            ship1_type == TYPE_B || ship1_type == TYPE_C || ship1_type == TYPE_D,
            "ShipFactory: material is not allow type"
        );

        let token_id = gen_token_id(&ShipElements {
            prefix_id: self.next_id,
            ship_type: ship1_type + 1,
            ship_sub_type: target_sub_type,
            capacity: self.internal_random_spaceship_capacity(&(ship1_type + 1)),
        });
        require!(
            self.tokens.owner_by_id.get(&token_id).is_none(),
            "ERR_TOKEN_ID_CONFLICT"
        );
        self.next_id += 1;

        self.mint_ship_with_supply_updated(owner_id.clone(), &token_id);

        ext_shippool::register_ship(
            token_id.clone(),
            owner_id.clone(),
            self.shippool_id.clone(),
            0,
            GAS_FOR_REGISTER_SHIP,
        );

        self.burn_eng_for_user(owner_id.clone(), eng_amount);
        self.burn_ship_with_supply_updated(&owner_id, &token_id_1, None);
        self.burn_ship_with_supply_updated(&owner_id, &token_id_2, None);

        Event::UpgradeEvent {
            sender_id: &predecessor_id,
            token_id_1: &token_id_1,
            token_id_2: &token_id_2,
            token_id: &token_id,
        }
        .emit();
    }
}

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

impl Contract {
    pub(crate) fn gen_metadata(&self, token_id: &String) -> TokenMetadata {
        let ship_element = parse_token_id(token_id);
        let ship_sub_type = format!("{}:{}", ship_element.ship_type, ship_element.ship_sub_type);
        TokenMetadata {
            title: Some("Spaceship".to_string()),
            description: Some(token_id.clone()),
            media: self.icons.get(&ship_sub_type).and_then(|x| Some(x)),
            //Some(Base64VecU8::from("a".repeat(64).as_bytes().to_vec()))
            media_hash: None,
            copies: Some(1),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    pub(crate) fn upate_total_supply(&mut self, token_id: &String, delta: i8) {
        let ship_element = parse_token_id(&token_id);
        let ship_type = format!("{}", ship_element.ship_type);
        let ship_type_detail = format!("{}:{}", ship_element.ship_type, ship_element.ship_sub_type);

        // add type total supply
        let mut new_supply = self.total_supplies.get(&ship_type).unwrap_or(ShipSupply {
            active: 0,
            burned: 0,
        });
        new_supply.active = (new_supply.active as i64 + delta as i64) as u32;
        self.total_supplies.insert(&ship_type, &new_supply);

        // add type-detail total supply
        let mut new_supply = self
            .total_supplies
            .get(&ship_type_detail)
            .unwrap_or(ShipSupply {
                active: 0,
                burned: 0,
            });
        new_supply.active = (new_supply.active as i64 + delta as i64) as u32;
        self.total_supplies.insert(&ship_type_detail, &new_supply);
    }

    pub(crate) fn update_owner_supply(
        &mut self,
        token_id: &String,
        from: Option<AccountId>,
        to: Option<AccountId>,
    ) {
        let ship_element = parse_token_id(&token_id);
        let ship_type = format!("{}", ship_element.ship_type);
        let ship_type_detail = format!("{}:{}", ship_element.ship_type, ship_element.ship_sub_type);

        // transfer or burn
        if let Some(owner_id) = from {
            let mut owner_supplies = self.supply_per_owner.get(&owner_id).unwrap_or_else(|| {
                UnorderedMap::new(StorageKey::SupplyPerOwner {
                    account_hash: env::sha256(owner_id.as_bytes()),
                })
            });

            // update type owner supply
            let mut new_supply = owner_supplies.get(&ship_type).unwrap_or(ShipSupply {
                active: 0,
                burned: 0,
            });
            new_supply.active -= 1;
            if to.is_none() {
                new_supply.burned += 1;
            }
            owner_supplies.insert(&ship_type, &new_supply);
            // update type-detail owner supply
            let mut new_supply = owner_supplies.get(&ship_type_detail).unwrap_or(ShipSupply {
                active: 0,
                burned: 0,
            });
            new_supply.active -= 1;
            if to.is_none() {
                new_supply.burned += 1;
            }
            owner_supplies.insert(&ship_type_detail, &new_supply);

            self.supply_per_owner.insert(&owner_id, &owner_supplies);
        }

        // mint or transfer
        if let Some(owner_id) = to {
            let mut owner_supplies = self.supply_per_owner.get(&owner_id).unwrap_or_else(|| {
                UnorderedMap::new(StorageKey::SupplyPerOwner {
                    account_hash: env::sha256(owner_id.as_bytes()),
                })
            });

            // update type owner supply
            let mut new_supply = owner_supplies.get(&ship_type).unwrap_or(ShipSupply {
                active: 0,
                burned: 0,
            });
            new_supply.active += 1;
            owner_supplies.insert(&ship_type, &new_supply);
            // update type-detail owner supply
            let mut new_supply = owner_supplies.get(&ship_type_detail).unwrap_or(ShipSupply {
                active: 0,
                burned: 0,
            });
            new_supply.active += 1;

            owner_supplies.insert(&ship_type_detail, &new_supply);

            self.supply_per_owner.insert(&owner_id, &owner_supplies);
        }
    }

    pub fn mint_ship_with_supply_updated(&mut self, owner_id: AccountId, token_id: &String) {
        self.tokens
            .internal_mint(token_id.clone(), owner_id.clone());
        self.upate_total_supply(token_id, 1);
        self.update_owner_supply(token_id, None, Some(owner_id.clone()));
    }

    // SSP-00-10
    pub fn internal_random_spaceship_capacity(&self, ship_type: &u8) -> u32 {
        let mut type_capacity: CapacityRange = CapacityRange { min: 0, max: 0 };

        if *ship_type == TYPE_A {
            type_capacity = self.capacity.capacity_a.clone();
        }
        if *ship_type == TYPE_B {
            type_capacity = self.capacity.capacity_b.clone();
        }
        if *ship_type == TYPE_C {
            type_capacity = self.capacity.capacity_c.clone();
        }
        if *ship_type == TYPE_D {
            type_capacity = self.capacity.capacity_d.clone();
        }
        if *ship_type == TYPE_S {
            type_capacity = self.capacity.capacity_s.clone();
        }

        let mut min: u32 = type_capacity.min;
        let max: u32 = type_capacity.max;
        let base: u32 = max - min;

        if base > 0 {
            let rnd = self.internal_random();
            min += (rnd % base as u64) as u32;
        }
        min
    }

    // only time lock to set
    // SSP-00-11
    pub fn set_spaceship_type_capacity(&mut self, ship_type: u8, min: u32, max: u32) {
        require!(max >= min, "SpaceShip: invalid min and max");

        if ship_type == TYPE_A {
            self.capacity.capacity_a.min = min;
            self.capacity.capacity_a.max = max;
        }
        if ship_type == TYPE_B {
            self.capacity.capacity_b.min = min;
            self.capacity.capacity_b.max = max;
        }
        if ship_type == TYPE_C {
            self.capacity.capacity_c.min = min;
            self.capacity.capacity_c.max = max;
        }
        if ship_type == TYPE_D {
            self.capacity.capacity_d.min = min;
            self.capacity.capacity_d.max = max;
        }
        if ship_type == TYPE_S {
            self.capacity.capacity_s.min = min;
            self.capacity.capacity_s.max = max;
        }
    }

    // SSP-00-01
    pub fn burn_ship_with_supply_updated(
        &mut self,
        owner_id: &AccountId,
        token_id: &TokenId,
        eng_receiver: Option<AccountId>,
    ) {
        // burn token
        let ship_element = parse_token_id(token_id);
        require!(
            ship_element.ship_type != TYPE_S,
            "SpaceShip: not support type"
        );
        self.tokens.internal_burn(token_id);

        self.upate_total_supply(&token_id, -1);
        self.update_owner_supply(&token_id, Some(owner_id.clone()), None);

        // send eng
        if let Some(to) = eng_receiver {
            self.internal_mint_eng_for_user(
                to,
                U128(self.eng_type_reward[(ship_element.ship_type - 1) as usize]),
            );
        }
    }

    pub fn internal_random(&self) -> u64 {
        // get random data
        let seeds: Vec<u8> = env::random_seed();
        let mut seed: u128 = 0;

        for i in 0..4 {
            let mut temp_seed: u128 = 0;
            for j in 0..8 {
                temp_seed = temp_seed * 10 + seeds[i * 4 + j] as u128;
            }
            seed += temp_seed;
        }

        let mut r = StdRng::seed_from_u64(seed as u64); // <- Here we set the seed
        let rnd: u64 = r.gen();

        rnd // random number
    }
}
