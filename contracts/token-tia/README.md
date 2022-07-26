
# token-tia
This is a NEP-141 compatible token contract.
### deploy and init
```bash
near deploy $TOKEN releases/token_tia_release.wasm --account_id=$TOKEN
near call $TOKEN new '{"owner_id": "'$OWNER'", "name": "Theia", "symbol": "TIA", "decimals": 18}' --account_id=$TOKEN
```
### custom interfaces
```bash
# get version and owner info
near view $TOKEN get_metadata 

# directly burn 1000 
near call $TOKEN burn '{"amount": "1000"}' --depositYocto=1 --account_id=alice.testnet

# transfer 1000 to bob and burn 500
near call $TOKEN batch_transfer '{"receiver_ids": ["bob.testnet", ""], "amounts": ["1000", "500"]}' --depositYocto=1 --account_id=alice.testnet

# owner can transfer ownership to other account
near call $TOKEN set_owner '{"owner_id": "bob.testnet"}' --depositYocto=1 --account_id=alice.testnet

# owner can mint token to owner itself
near call $TOKEN mint '{"amount": "1000"}' --depositYocto=1 --account_id=alice.testnet

# owner can set token metadata
near call $TOKEN set_token_meta '{"name": "Theia", "symbol": "TIA", "dec": 18}' --depositYocto=1 --account_id=alice.testnet

# owner can set token icon
near call $TOKEN set_icon '{"icon": "xxxxxx"}' --depositYocto=1 --account_id=alice.testnet
```

### NEP-141 interfaces
All interfaces are supported, for details, see:
[NEP-141](https://nomicon.io/Standards/Tokens/FungibleToken/)

### NEP-145 interfaces
All interfaces are supported, for details, see:
[NEP-145](https://nomicon.io/Standards/StorageManagement)