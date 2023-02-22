# NFT-Staking pallet

A pallet used to stake non-fungible assets in a so called 'Staking Contract'

## Overview

The NFT-Staking pallet provides functionality for the interaction with 'Staking Contract' instances, including:

* Funding reward treasury
* Contract creation
* Contract taking
* Redemption of finished contracts

To use it in your runtime, you need to implement [`pallet_ajuna_nft_staking::Config`](https://github.com/ajuna-network/Ajuna/blob/develop/pallets/nft-staking/src/lib.rs#L88).

The supported dispatchable functions are documented in the [`pallet_ajuna_nft_staking::Call`](https://github.com/ajuna-network/Ajuna/blob/develop/pallets/nft-staking/src/lib.rs#L324) enum.

### Staking Contracts

The type definitions for all elements of the staking contracts can be seen [here](https://github.com/ajuna-network/Ajuna/blob/develop/pallets/nft-staking/src/contracts.rs).

Contracts defined by three elements: 

1. The contract clauses, used to validate if the account that wishes to take the contract is able to do so or not.
2. The duration in blocks, which is the amount of time the contract will remain active once taken, after that the reward can be claimed.
3. The contract reward, which is awarded to the account that took it after the contract finishes its active period.

When an account submits a staking contract, the contract rewards defined in the contract will be taken from that account and stored under the custody of the pallet's treasury account. At the moment
a given account wants to take the contract, all their staked assets will also be, in this case, temporarily claimed by the treasury account, until the contract is ready to be redeemed.

#### Contract clauses

Contract clauses are a list of criteria that need to be fulfilled for the contract to be taken by anyone. These clauses are related to other non-fungible assets which
the taker **must** stake in order for the contract to be awarded to them.

Currently, the clauses are limited to these two options:

* `HasAttribute`: Checks if the given non-fungible asset has the given attribute in the specified namespace.
* `HasAttributeWithValue`: Checks if the given non-fungible asset has the given attribute in the specified namespace with the specified value.

Contract clauses are evaluated in order, that means that **contract clause 1 will be checked with the asset in the stake vector at position 1**, take that into account
when building your logic.

The non-fungible asset provider should be compatible with the [`frame_support::traits::tokens::nonfungibles_v2`](https://docs.rs/frame-support/14.0.0/frame_support/traits/tokens/nonfungibles_v2/index.html) specification.

#### Contract duration

Once a contract has been taken by a given account, the contract enters it's 'Active' state, during that time no interactions can happen with it, after the amount of blocks specified in the contract 
has passed the contract **can then only be redeemed by the same account that took it**.

#### Contract reward

Contracts can have one of this two types of rewards:

* `Token`: The contract taker will be awarded the defined amount of tokens in their contract upon completion, this amount will come from the original deposit the contract creator was forced to make.
* `NFT`: The contract taker will be awarded a given non-fungible asset, this asset would have been previously owned by the contract creator.

## Good to know

* The `nft-staking` pallet works mostly through the `NFTHelper` type, which means that whatever pallet you use in its configuration will need to be properly secured and configured, otherwise
the `nft-staking` pallet may behave unexpectedly. For example: if a reward nft is transferred between accounts before being awarded.
* The contract clauses are currently limited to a maximum of 10, keep that in mind while designing your logic.
* The `nonfungibles_v2` although being a good basis, does have some limitations that the implementor pallets try to fix like [`pallet-nfts`](https://github.com/paritytech/substrate/tree/polkadot-v0.9.37/frame/nfts),
so it's important to familiarise yourself with the choice of pallet you pick for asset storage and management.

