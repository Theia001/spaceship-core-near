use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::TokenId;

use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, require, AccountId, BorshStorageKey, Gas,
    PanicOnDefault, PromiseOrValue,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

mod events;
mod ft;
mod mynft;
mod nft;
mod owner;
mod view;

pub use crate::events::*;
pub use crate::ft::*;
pub use crate::nft::*;
pub use crate::owner::*;
pub use crate::view::*;
use mynft::MyNonFungibleToken;

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_REGISTER_SHIP: Gas = Gas(70 * TGAS);
pub const GAS_FOR_TRANSFER: Gas = Gas(25 * TGAS);
pub const YOCTO18: u128 = 1_000_000_000_000_000_000;
pub const MAX_ICON_LENGTH: usize = 2048;

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct CapacityRange {
    min: u32,
    max: u32,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Capacity {
    capacity_a: CapacityRange,
    capacity_b: CapacityRange,
    capacity_c: CapacityRange,
    capacity_d: CapacityRange,
    capacity_s: CapacityRange,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct ShipSupply {
    active: u32,
    burned: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Debug, C&lone))]
pub struct ShipElements {
    prefix_id: u64,
    ship_type: u8,
    ship_sub_type: u8,
    capacity: u32,
}

impl ShipElements {
    pub fn to_token_id(&self) -> TokenId {
        format!(
            "{}:{}:{}:{}",
            self.prefix_id, self.ship_type, self.ship_sub_type, self.capacity
        )
    }
}

#[ext_contract(ext_shippool)]
pub trait ShipPool {
    fn register_ship(&mut self, token_id: String, token_owner_id: AccountId);
    fn batch_register_ships(&mut self, token_ids: Vec<String>, token_owner_id: AccountId);
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    NonFungibleToken,
    Metadata,
    Enumeration,
    TokensPerOwner { account_hash: Vec<u8> },
    Supply,
    SupplyPerOwner { account_hash: Vec<u8> },
    TotalSupply,
    Icon,

    Eng,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    eng: FungibleToken,
    eng_icon: Option<String>,

    tokens: MyNonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    next_id: u64,
    // ship_type or ship_type:ship_sub_type, to ShipSupply
    total_supplies: UnorderedMap<String, ShipSupply>,
    supply_per_owner: LookupMap<AccountId, UnorderedMap<String, ShipSupply>>,
    // ship_type:ship_sub_type, to ShipSupply
    icons: UnorderedMap<String, String>,

    box_id: AccountId,
    shippool_id: AccountId,
    shipmarket_id: AccountId,
    auction_id: AccountId,
    luckpool_id: AccountId,

    // type => array  arr[0] is min and arr[1] is max.
    capacity: Capacity,

    // eng_type_reward
    eng_type_reward: Vec<u128>, // TYPE_S, TYPE_A, TYPE_B, TYPE_C, TYPE_D
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        box_id: AccountId,
        shippool_id: AccountId,
        shipmarket_id: AccountId,
        auction_id: AccountId,
        luckpool_id: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut contract = Contract {
            owner_id: owner_id.clone().into(),
            eng: FungibleToken::new(StorageKey::Eng),
            eng_icon: None,
            tokens: MyNonFungibleToken::new(owner_id),
            metadata: LazyOption::new(
                StorageKey::Metadata,
                Some(&NFTContractMetadata {
                    spec: NFT_METADATA_SPEC.to_string(),
                    name: "Theia Spaceship NFT".to_string(),
                    symbol: "tieships".to_string(),
                    icon: None,
                    base_uri: None,
                    reference: None,
                    reference_hash: None,
                }),
            ),
            next_id: 1_u64,
            total_supplies: UnorderedMap::new(StorageKey::TotalSupply),
            supply_per_owner: LookupMap::new(StorageKey::Supply),
            icons: UnorderedMap::new(StorageKey::Icon),

            box_id,
            shippool_id,
            shipmarket_id,
            auction_id,
            luckpool_id,

            capacity: Capacity {
                capacity_a: CapacityRange { min: 80, max: 120 },
                capacity_b: CapacityRange { min: 40, max: 60 },
                capacity_c: CapacityRange { min: 20, max: 30 },
                capacity_d: CapacityRange { min: 10, max: 15 },
                capacity_s: CapacityRange { min: 240, max: 240 },
            },

            // TYPE_S, TYPE_A, TYPE_B, TYPE_C, TYPE_D. Notice: the index is reversed
            eng_type_reward: vec![YOCTO18, 2 * YOCTO18, 4 * YOCTO18, 8 * YOCTO18, 20 * YOCTO18],
        };
        // mint 4 S-class spaceship
        for i in 1..(1 + TYPE_S_COUNT) {
            contract.mint_ship_with_supply_updated(
                env::current_account_id(),
                &ShipElements {
                    prefix_id: contract.next_id,
                    ship_type: TYPE_S,
                    ship_sub_type: i as u8,
                    capacity: 240,
                }
                .to_token_id(),
            );
            contract.next_id += 1;
        }

        contract
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, eng);
near_contract_standards::impl_fungible_token_storage!(Contract, eng);
