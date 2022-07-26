use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use mynft::MyNonFungibleToken;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedSet, LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault,
    PromiseOrValue, Gas, ext_contract, Promise, log
};
use rand_distr::{Normal, Distribution};
use rand::{Rng,SeedableRng};
use rand::rngs::StdRng;

mod view;
mod owner;
mod mynft;
mod utils;
mod events;

pub use crate::utils::*;
pub use crate::events::*;
// 
pub type BoxType = u8;
pub type AccountIdIndex = String;
// box type
pub const TYPE_U: u8 = 1;
pub const TYPE_S: u8 = 2;
// ship type
pub const TYPE_SHIP_S: u8 = 5;
pub const TYPE_SHIP_A: u8 = 4;
pub const TYPE_SHIP_B: u8 = 3;
pub const TYPE_SHIP_C: u8 = 2;
pub const TYPE_SHIP_D: u8 = 1;

// reward basic
pub const RATE_DENOMINATOR: u8 = 100;
// prob basic
pub const PROB_DENOMINATOR: u64 = 10000;


#[derive(BorshSerialize, BorshDeserialize)]
pub struct Open {
    token_id: TokenId,
    ship_type: u8,
    ship_sub_type: u8,
    capacity: u32,
}

#[derive(Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct MagicBox {
    token_id: TokenId,
    box_type: u8,
}

#[derive(BorshDeserialize, BorshSerialize,Clone)]
pub struct ProbInfo {
    box_u_ship_a: u32,
    box_u_ship_b: u32,
    box_u_ship_c: u32,
    box_u_ship_d: u32,
    box_s_ship_a: u32,
    box_s_ship_b: u32,
    box_s_ship_c: u32,
    box_s_ship_d: u32,
}

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_MINT_SHIP: Gas = Gas(180 * TGAS);
#[ext_contract(ext_spaceship)]
pub trait Spaceship {
    fn batch_mint(
        &mut self,
        owner_id: AccountId,
        ship_types: Vec<String>,
        ship_sub_types: Vec<String>,
    );
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    TokensPerOwner { account_hash: Vec<u8> },
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    TypeBalance,
    TypeBurnBalance,
    OwnedTokens,
    OwnedTokensIndex,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,

    //tokens: NonFungibleToken,
    tokens: MyNonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    spaceship_contract_id: AccountId,
    next_token_idx: u64,
    
    shift: usize, 
    // numLimit.  
    num_limit: u8, //default: 10,

    // burnBalance
    burn_balance: u64,


    // key is type return num which num does not contain burned num. typeBalance
    type_balance: UnorderedMap<BoxType, u128>, 
    // key is type return num which num is burned num. typeBurnBalance
    type_burn_balance: UnorderedMap<BoxType, u128>,

    // Mapping from owner to list of owned token IDs. _ownedTokens
    owned_tokens: UnorderedMap<AccountIdIndex, TokenId>, 
    //mapping(address => mapping(uint256 => uint256)) private _ownedTokens;

    // Mapping from token ID to index of the owner tokens list. _ownedTokensIndex
    owned_tokens_index: UnorderedMap<TokenId, u64>, 
    
    // type_prob
    type_prob: ProbInfo,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, spaceship_contract_id: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Contract {
            owner_id: owner_id.clone().into(),
            tokens: MyNonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(
                StorageKey::Metadata,
                Some(&NFTContractMetadata {
                    spec: NFT_METADATA_SPEC.to_string(),
                    name: "Theia Magicbox NFT".to_string(),
                    symbol: "tieboxes".to_string(),
                    icon: None,
                    base_uri: None,
                    reference: None,
                    reference_hash: None,
                }),
            ),
            spaceship_contract_id,
            next_token_idx: 0,
            shift: 0,
            num_limit:  5, // default: 10. Currently set to 5. keep same value as boxmall
            burn_balance: 0,

            // key is type return num which num does not contain burned num.
            type_balance: UnorderedMap::new(StorageKey::TypeBalance),
            // key is type return num which num is burned num.
            type_burn_balance: UnorderedMap::new(StorageKey::TypeBurnBalance),

            owned_tokens: UnorderedMap::new(StorageKey::OwnedTokens),

            owned_tokens_index: UnorderedMap::new(StorageKey::OwnedTokensIndex),

            // type_prob
            type_prob: ProbInfo{
                box_u_ship_a: 500,
                box_u_ship_b: 1000,
                box_u_ship_c: 2500,
                box_u_ship_d: 6000,
                box_s_ship_a: 100,
                box_s_ship_b: 400,
                box_s_ship_c: 1500,
                box_s_ship_d: 8000},
        }
 
    }

    // MBX-00-02
    #[payable]
    pub fn open_box(&mut self, token_ids: Vec<TokenId>) {
        assert_one_yocto();
        require!(token_ids.len() > 0 && token_ids.len() <= self.num_limit.into(), "invalid open num");
        let owner_id = env::predecessor_account_id();

        let mut ship_types: Vec<String> = vec![];
        let mut ship_sub_types: Vec<String> = vec![];
        let mut open_list: Vec<Open> = vec![];

        for i in 0..token_ids.len() {
            let token_id: TokenId = token_ids[i].clone();
            let token = self.tokens.nft_token(token_id.clone()).expect("ERR_BOX_NOT_EXIST");
            require!(token.owner_id==owner_id, "ERR_BOX_NOT_EXIST");
    
            let box_type: BoxType = self.get_type_by_token_id(token_id.clone());

            
            // get random data. 
            let rnd: u64 = self.internal_random();
            let ship_type = format!("{}", self.internal_random_spaceship_type(rnd, box_type));
            let ship_sub_type = format!("{}", self.internal_random_spaceship_subtype(ship_type.parse::<u8>().unwrap()));

            ship_types.push(ship_type);
            ship_sub_types.push(ship_sub_type);
    
            self.internal_burn(token_id.clone());
        }

        // cross-contract call mint spaceship
        ext_spaceship::batch_mint(
            owner_id.clone(),
            ship_types,
            ship_sub_types,
            self.spaceship_contract_id.clone(),
            0,
            GAS_FOR_MINT_SHIP,
        );

        Event::OpenBox {
            caller_id: &owner_id.clone(),
            num: token_ids.len() as u64,
            token_ids,
        }.emit();

    }

    // MBX-00-03
    pub fn get_box_list(&self, user: AccountId, page: u32, size: u32) -> (u32, Vec<MagicBox>) {
        let tokens_for_owner = self.nft_tokens_for_owner(user,  Some(U128(page as u128)), Some(size as u64));
        let total: u32 = tokens_for_owner.len() as u32;
        let mut box_list: Vec<MagicBox> = vec![];
        for item in tokens_for_owner.iter(){
            // get box_type from token_id
            let box_type: u8 = self.get_type_by_token_id(item.token_id.clone());
            let box_node: MagicBox = MagicBox{token_id: item.token_id.clone(),box_type: box_type};
            box_list.push(box_node);
        }
        return (total, box_list);
    }
    //

    // MBX-00-01
    pub fn mint_nft(&mut self, 
        box_type: BoxType,
        token_owner_id: AccountId,
        token_metadata: Option<TokenMetadata>,
    ) -> Token {
        let token_id: TokenId = format!("{}:{}", self.next_token_idx, box_type);
        self.next_token_idx +=1;

        if self.tokens.token_metadata_by_id.is_some() && token_metadata.is_none() {
            env::panic_str("Must provide metadata");
        }
        if self.tokens.owner_by_id.get(&token_id).is_some() {
            env::panic_str("token_id must be unique");
        }

        let owner_id: AccountId = token_owner_id.clone().into();

        // Core behavior: every token must have an owner
        self.tokens.owner_by_id.insert(&token_id, &owner_id);

        // Metadata extension: Save metadata, keep variable around to return later.
        // Note that check above already panicked if metadata extension in use but no metadata
        // provided to call.
        self.tokens.token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, &token_metadata.as_ref().unwrap()));

        // Enumeration extension: Record tokens_per_owner for use with enumeration view methods.
        let mut length: u64 = 0;
        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::TokensPerOwner {
                    account_hash: env::sha256(owner_id.as_bytes()),
                })
            });
            length = token_ids.len();
            token_ids.insert(&token_id);
            tokens_per_owner.insert(&owner_id, &token_ids);
        }
        
        let acc_index: AccountIdIndex = format!("{}:{}", token_owner_id.clone(), length);
        self.owned_tokens.insert(&acc_index,&token_id);
        self.owned_tokens_index.insert(&token_id,&length);

        Token { token_id, owner_id, metadata: token_metadata, approved_account_ids: None }
    }

    // MBX-00-04
    pub fn batch_mint(&mut self, 
        token_owner_id: AccountId,
        box_type: BoxType,
        num: u32,
    ) {
        let predecessor_id = env::predecessor_account_id();

        for _i in 0..num {
            let token_id: TokenId = format!("{}:{}", self.next_token_idx, box_type);
            self.next_token_idx +=1;

            if self.tokens.owner_by_id.get(&token_id).is_some() {
                env::panic_str("token_id must be unique");
            }

            let owner_id: AccountId = token_owner_id.clone().into();

            // Core behavior: every token must have an owner
            self.tokens.owner_by_id.insert(&token_id, &owner_id);

            // Metadata extension: Save metadata, keep variable around to return later.
            // Note that check above already panicked if metadata extension in use but no metadata
            // provided to call.

            let token_metadata: Option<TokenMetadata> = Some(TokenMetadata {
                title: Some("NFT MAGICBOX".to_string()),
                description: Some("NFT MAGICBOX".to_string()),
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

            self.tokens.token_metadata_by_id
                .as_mut()
                .and_then(|by_id| by_id.insert(&token_id, &token_metadata.as_ref().unwrap()));

            // Enumeration extension: Record tokens_per_owner for use with enumeration view methods.
            let mut length: u64 = 0;
            if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
                let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                    UnorderedSet::new(StorageKey::TokensPerOwner {
                        account_hash: env::sha256(owner_id.as_bytes()),
                    })
                });
                length = token_ids.len();
                token_ids.insert(&token_id);
                tokens_per_owner.insert(&owner_id, &token_ids);
            }
            
            //
            let acc_index: AccountIdIndex = format!("{}:{}", token_owner_id.clone(), length);
            self.owned_tokens.insert(&acc_index,&token_id);
            self.owned_tokens_index.insert(&token_id,&length);
            //

        }

        //Token { token_id, owner_id, metadata: token_metadata, approved_account_ids: None }
    }
 
    // MBX-00-05
    pub fn internal_burn(&mut self, token_id: TokenId ) {
        self.nft_transfer(AccountId::new_unchecked("".to_string()),token_id,None,None);
    }

}

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}


#[near_bindgen]
impl Contract {
 
    // MBX-00-07
    pub fn set_spaceship_type_prob(&mut self, box_type: BoxType, prob_a: u32, prob_b: u32, prob_c: u32, prob_d: u32) {
        require!(u64::from( prob_a + prob_b + prob_c + prob_d ) > PROB_DENOMINATOR, "MagicBox: prob is not enough");
        require!( box_type == TYPE_U || box_type == TYPE_S, "MagicBox: type is invalid");

        if box_type == TYPE_U {
            self.type_prob.box_u_ship_a = prob_a;
            self.type_prob.box_u_ship_b = prob_b;
            self.type_prob.box_u_ship_c = prob_c;
            self.type_prob.box_u_ship_d = prob_d;
        }
        if box_type == TYPE_S {
            self.type_prob.box_s_ship_a = prob_a;
            self.type_prob.box_s_ship_b = prob_b;
            self.type_prob.box_s_ship_c = prob_c;
            self.type_prob.box_s_ship_d = prob_d;
        }       
        
        // emit event...
        
    }

    // MBX-00-08
    pub fn balance_of( &self, account_id: AccountId ) -> u32 {
        let tokens_for_owner = self.nft_tokens_for_owner(account_id,  Some(U128(0)), Some(10000000 as u64));
        let total: u32 = tokens_for_owner.len() as u32;
        total
    }

    // MBX-00-09
    pub fn balance_type_of( &self, account_id: AccountId, box_type: BoxType ) -> u32 {
        let tokens_for_owner = self.nft_tokens_for_owner(account_id,  Some(U128(0)), Some(10000000 as u64));
         
        let mut total: u32 = 0;
        for item in tokens_for_owner.iter(){
            // get box_type from token_id
            if box_type == self.get_type_by_token_id(item.token_id.clone()) {
                total += 1
            }
        }
        total
    }

    // MBX-00-10
    pub fn owner_of( &mut self, token_id: TokenId ) -> AccountId {
        self.tokens.owner_by_id.get(&token_id).unwrap()
    }   

    // MBX-00-11
    pub fn get_type_by_token_id( &self, token_id: TokenId ) -> BoxType {
        let items :Vec<&str> = token_id.split(":").collect();    
        let box_type: u8 = items[1].parse::<u8>().unwrap(); // token format: "x:y"
        box_type
    }

    // tokenOfOwnerByIndex
    // MBX-00-12
    pub fn token_of_owner_by_index( &self, account_id: AccountId, index: u64) -> TokenId {
        let acc_index: AccountIdIndex = format!("{}:{}", account_id, index);
        self.owned_tokens.get(&acc_index).unwrap()
    }

    // MBX-00-13
    pub fn token_type_of_owner_by_index( &self, account_id: AccountId, index: u64) -> TokenId {
        let acc_index: AccountIdIndex = format!("{}:{}", account_id, index);
        self.owned_tokens.get(&acc_index).unwrap()
    }

    // MBX-00-14
    pub fn set_num_limit( &mut self, limit: u8) {
        self.num_limit = limit;
    }

    // MBX-00-15
    pub fn internal_random_spaceship_subtype( &mut self, ship_type: u8 ) -> u8{
        let mut sub_type: u8 = 0;
        let rnd: u64 = self.internal_random();
        if ship_type == TYPE_SHIP_A {
            sub_type = 1 + (rnd % 4) as u8; // 4 subtypes for A
        }
        if ship_type == TYPE_SHIP_B {
            sub_type = 1 + (rnd % 8) as u8; // 8 subtypes for B
        }
        if ship_type == TYPE_SHIP_C {
            sub_type = 1 + (rnd % 16) as u8; // 16 subtypes for C
        }
        if ship_type == TYPE_SHIP_D {
            sub_type = 1 + (rnd % 32) as u8; // 32 subtypes for D
        }
        sub_type
    }

    pub fn internal_random( &mut self ) -> u64 {
        // get random data
        let seeds: Vec<u8> = env::random_seed();
        let mut seed: u128 = 0;

        if self.shift > 24 {
            self.shift = 0;
        }

        for i in self.shift..self.shift+8{
            seed = seed * 10 + seeds[i] as u128;
        }
        self.shift += 2;

        let mut r = StdRng::seed_from_u64(seed as u64); // <- Here we set the seed
        let rnd: u64 = r.gen();

        rnd // random number
    }
}

impl Contract {
   // MBX-00-06
   pub fn internal_random_spaceship_type( &self, random: u64, box_type: BoxType) -> u8 {
      let prob = random % PROB_DENOMINATOR;
      let mut ship_type = 0;
      let mut block_a = 0;
      let mut block_b = 0;
      let mut block_c = 0;
      let mut block_d = 0;

      if box_type == TYPE_U {
          block_a = self.type_prob.box_u_ship_a;
          block_b = self.type_prob.box_u_ship_b + block_a;
          block_c = self.type_prob.box_u_ship_c + block_b;
          block_d = self.type_prob.box_u_ship_d + block_c;
      }
      if box_type == TYPE_S {
          block_a = self.type_prob.box_s_ship_a;
          block_b = self.type_prob.box_s_ship_b + block_a;
          block_c = self.type_prob.box_s_ship_c + block_b;
          block_d = self.type_prob.box_s_ship_d + block_c;
      }
    
      if prob < block_a.into() {
          ship_type = TYPE_SHIP_A;
      }
      if u64::from(block_a) <= prob && prob < block_b.into() {
          ship_type = TYPE_SHIP_B;
      }
      if u64::from(block_b) <= prob && prob < block_c.into() {
          ship_type = TYPE_SHIP_C;
      }

      if u64::from(block_c) <= prob && prob < block_d.into() {
          ship_type = TYPE_SHIP_D;
      }

      ship_type
   }
}