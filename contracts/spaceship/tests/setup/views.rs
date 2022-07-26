use crate::setup::*;

impl Env {
    pub fn get_metadata(&self) -> Metadata {
        self.owner
            .view_method_call(self.spaceship.contract.get_metadata())
            .unwrap_json::<Metadata>()
    }

    pub fn get_eng_balance_of(&self, account_id: AccountId) -> U128 {
        self.owner
            .view_method_call(self.spaceship.contract.ft_balance_of(account_id))
            .unwrap_json::<U128>()
    }

    pub fn get_eng_metadata(&self) -> FungibleTokenMetadata {
        self.owner
            .view_method_call(self.spaceship.contract.ft_metadata())
            .unwrap_json::<FungibleTokenMetadata>()
    }

    pub fn get_ship_icon(&self, type_detail: String) -> Option<String> {
        self.owner
            .view_method_call(self.spaceship.contract.get_ship_icon(type_detail))
            .unwrap_json::<Option<String>>()
    }

    pub fn get_spaceship_supply(&self) -> SpaceShipSupply {
        self.owner
            .view_method_call(self.spaceship.contract.get_spaceship_supply())
            .unwrap_json::<SpaceShipSupply>()
    }

    pub fn get_spaceship_list_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<ShipElements> {
        self.owner
            .view_method_call(
                self.spaceship
                    .contract
                    .get_spaceship_list_for_owner(account_id, from_index, limit),
            )
            .unwrap_json::<Vec<ShipElements>>()
    }

    pub fn get_balance_type_of(&self, account_id: AccountId, ship_type: u8) -> u32 {
        self.owner
            .view_method_call(self.spaceship.contract.balance_type_of(account_id, ship_type))
            .unwrap_json::<u32>()
    }

    pub fn get_balance_subtype_of(&self, account_id: AccountId, ship_type: u8, ship_sub_type: u8) -> u32 {
        self.owner
            .view_method_call(self.spaceship.contract.balance_subtype_of(account_id, ship_type, ship_sub_type))
            .unwrap_json::<u32>()
    }
}
