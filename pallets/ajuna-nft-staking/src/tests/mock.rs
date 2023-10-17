// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{self as pallet_nft_staking, *};
use frame_support::{
	parameter_types,
	traits::{tokens::nonfungibles_v2::Create, AsEnsureOriginWithArg, ConstU16, ConstU64, Hooks},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_nfts::{CollectionSettings, ItemConfig, ItemSettings, PalletFeatures};
use sp_core::bounded_vec;
use sp_runtime::{
	testing::{TestSignature, H256},
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};

pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockNonce = u64;
pub type MockBalance = u64;

pub type CurrencyOf<T> = <T as Config>::Currency;
pub type NftHelperOf<T> = <T as Config>::NftHelper;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;

pub const RESERVED_COLLECTION_0: MockCollectionId = 0;
pub const RESERVED_COLLECTION_1: MockCollectionId = 1;
pub const RESERVED_COLLECTION_2: MockCollectionId = 2;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system,
		Balances: pallet_balances,
		Nft: pallet_nfts,
		NftStake: pallet_nft_staking,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = MockAccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<MockBalance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type Nonce = MockNonce;
	type Block = MockBlock;
}

parameter_types! {
	pub const MockExistentialDeposit: MockBalance = 3;
}

impl pallet_balances::Config for Test {
	type Balance = MockBalance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = MockExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
}

pub type MockCollectionId = u32;
pub type MockItemId = H256;

parameter_types! {
	pub const CollectionDeposit: MockBalance = 333;
	pub const ItemDeposit: MockBalance = 33;
	pub const MetadataDepositBase: MockBalance = 0;
	pub const AttributeDepositBase: MockBalance = 0;
	pub const DepositPerByte: MockBalance = 0;
	pub const StringLimit: u32 = 128;
	pub const ApprovalsLimit: u32 = 1;
	pub const ItemAttributesApprovalsLimit: u32 = 10;
	pub const MaxTips: u32 = 1;
	pub const MaxDeadlineDuration: u32 = 1;
	pub ConfigFeatures: PalletFeatures = PalletFeatures::all_enabled();
}

#[cfg(feature = "runtime-benchmarks")]
pub struct Helper;
#[cfg(feature = "runtime-benchmarks")]
impl<CollectionId: From<u16>, ItemId: From<[u8; 32]>>
	pallet_nfts::BenchmarkHelper<CollectionId, ItemId> for Helper
{
	fn collection(i: u16) -> CollectionId {
		i.into()
	}
	fn item(i: u16) -> ItemId {
		let mut id = [0_u8; 32];
		let bytes = i.to_be_bytes();
		id[0] = bytes[0];
		id[1] = bytes[1];
		id.into()
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct ParameterGet<const N: u32>;

impl<const N: u32> Get<u32> for ParameterGet<N> {
	fn get() -> u32 {
		N
	}
}

pub type KeyLimit = ParameterGet<32>;
pub type ValueLimit = ParameterGet<64>;

impl pallet_nfts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = MockCollectionId;
	type ItemId = MockItemId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<MockAccountId>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<MockAccountId>>;
	type Locker = ();
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = StringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type ApprovalsLimit = ApprovalsLimit;
	type ItemAttributesApprovalsLimit = ItemAttributesApprovalsLimit;
	type MaxTips = MaxTips;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = ConstU32<2>;
	type Features = ConfigFeatures;
	type OffchainSignature = MockSignature;
	type OffchainPublic = MockAccountPublic;
	pallet_nfts::runtime_benchmarks_enabled! {
		type Helper = Helper;
	}
	type WeightInfo = ();
}

parameter_types! {
	pub const NftStakingPalletId: PalletId = PalletId(*b"aj/nftst");
	pub const MaxContracts: u32 = 5;
	pub const MaxStakingClauses: u32 = 10;
	pub const MaxFeeClauses: u32 = 1;
	pub const MaxMetadataLenght: u32 = 100;
	pub const AttributeMaxBytes: u32 = 32;
}

pub type CollectionConfig =
	pallet_nfts::CollectionConfig<MockBalance, BlockNumberFor<Test>, MockCollectionId>;

impl pallet_nft_staking::Config for Test {
	type PalletId = NftStakingPalletId;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CollectionId = MockCollectionId;
	type ItemId = MockItemId;
	type ItemConfig = pallet_nfts::ItemConfig;
	type NftHelper = Nft;
	type MaxContracts = MaxContracts;
	type MaxStakingClauses = MaxStakingClauses;
	type MaxFeeClauses = MaxFeeClauses;
	type MaxMetadataLength = MaxMetadataLenght;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type WeightInfo = ();
}

impl Default for RewardOf<Test> {
	fn default() -> Self {
		Reward::Tokens(Default::default())
	}
}
impl Default for ContractOf<Test> {
	fn default() -> Self {
		Contract {
			activation: Default::default(),
			active_duration: 1,
			claim_duration: 1,
			stake_duration: Default::default(),
			stake_clauses: Default::default(),
			fee_clauses: Default::default(),
			burn_fees: Default::default(),
			rewards: Default::default(),
			cancel_fee: Default::default(),
			nft_stake_amount: 1,
			nft_fee_amount: 1,
		}
	}
}
impl ContractOf<Test> {
	pub fn activation(mut self, activation: BlockNumberFor<Test>) -> Self {
		self.activation = Some(activation);
		self
	}
	pub fn active_duration(mut self, active_duration: BlockNumberFor<Test>) -> Self {
		self.active_duration = active_duration;
		self
	}
	pub fn claim_duration(mut self, claim_duration: BlockNumberFor<Test>) -> Self {
		self.claim_duration = claim_duration;
		self
	}
	pub fn stake_duration(mut self, stake_duration: BlockNumberFor<Test>) -> Self {
		self.stake_duration = stake_duration;
		self
	}
	pub fn stake_clauses(mut self, ns: AttributeNamespace, clauses: Vec<(u8, MockClause)>) -> Self {
		self.stake_clauses = clauses
			.into_iter()
			.map(|(target_index, clause)| MockContractClause {
				namespace: ns,
				target_index,
				clause,
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap();
		self
	}
	pub fn fee_clauses(mut self, ns: AttributeNamespace, clauses: Vec<(u8, MockClause)>) -> Self {
		self.fee_clauses = clauses
			.into_iter()
			.map(|(target_index, clause)| MockContractClause {
				namespace: ns,
				target_index,
				clause,
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap();
		self
	}
	pub fn rewards(mut self, rewards: BoundedRewardsOf<Test>) -> Self {
		self.rewards = rewards;
		self
	}
	pub fn cancel_fee(mut self, cancel_fee: MockBalance) -> Self {
		self.cancel_fee = cancel_fee;
		self
	}
	pub fn stake_amt(mut self, stake_amount: u8) -> Self {
		self.nft_stake_amount = stake_amount;
		self
	}
	pub fn fee_amt(mut self, fee_amount: u8) -> Self {
		self.nft_fee_amount = fee_amount;
		self
	}
}

pub type MockClause = Clause<MockCollectionId, KeyLimit, ValueLimit>;
pub type MockContractClause = ContractClause<MockCollectionId, KeyLimit, ValueLimit>;
pub struct MockClauses(pub Vec<MockClause>);
pub type MockMints =
	Vec<(NftId<MockCollectionId, MockItemId>, Attribute<KeyLimit>, Attribute<ValueLimit>)>;

impl From<MockClauses> for MockMints {
	fn from(clauses: MockClauses) -> Self {
		clauses
			.0
			.into_iter()
			.enumerate()
			.map(|(i, clause)| match clause {
				Clause::HasAttribute(collection_id, key) =>
					(NftId(collection_id, H256::random()), key, bounded_vec![i as u8]),
				Clause::HasAttributeWithValue(collection_id, key, value) =>
					(NftId(collection_id, H256::random()), key, value),
			})
			.collect()
	}
}

pub struct ExtBuilder {
	creator: Option<MockAccountId>,
	balances: Vec<(MockAccountId, MockBalance)>,
	create_contract_collection: bool,
	contract: Option<(MockItemId, ContractOf<Test>, bool)>,
	stakes: Vec<(MockAccountId, MockMints)>,
	fees: Vec<(MockAccountId, MockMints)>,
	accept_contract: Option<(MockItemId, MockAccountId)>,
	create_sniper: Option<(MockAccountId, ContractOf<Test>)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			creator: Default::default(),
			balances: vec![(ALICE, 999_999_999), (BOB, 999_999_999), (CHARLIE, 999_999_999)],
			create_contract_collection: Default::default(),
			contract: Default::default(),
			stakes: Default::default(),
			fees: Default::default(),
			accept_contract: Default::default(),
			create_sniper: Default::default(),
		}
	}
}

impl ExtBuilder {
	pub fn set_creator(mut self, creator: MockAccountId) -> Self {
		self.creator = Some(creator);
		self
	}
	pub fn balances(mut self, balances: Vec<(MockAccountId, MockBalance)>) -> Self {
		self.balances = balances;
		self
	}
	pub fn create_contract_collection(mut self) -> Self {
		self.create_contract_collection = true;
		self
	}
	pub fn create_contract(mut self, contract_id: MockItemId, contract: ContractOf<Test>) -> Self {
		self.contract = Some((contract_id, contract, false));
		self
	}
	pub fn create_contract_with_funds(
		mut self,
		contract_id: MockItemId,
		contract: ContractOf<Test>,
	) -> Self {
		self.contract = Some((contract_id, contract, true));
		self
	}
	pub fn mint_stakes(mut self, stakes: Vec<(MockAccountId, MockMints)>) -> Self {
		self.stakes = stakes;
		self
	}
	pub fn mint_fees(mut self, fees: Vec<(MockAccountId, MockMints)>) -> Self {
		self.fees = fees;
		self
	}
	pub fn accept_contract(
		mut self,
		stakes: Vec<(MockAccountId, MockMints)>,
		fees: Vec<(MockAccountId, MockMints)>,
		contract_id: MockItemId,
		by: MockAccountId,
	) -> Self {
		self.stakes = stakes;
		self.fees = fees;
		self.accept_contract = Some((contract_id, by));
		self
	}
	pub fn create_sniper(mut self, sniper: MockAccountId, contract: ContractOf<Test>) -> Self {
		self.create_sniper = Some((sniper, contract));
		self
	}
	pub fn build(self) -> sp_io::TestExternalities {
		let config = RuntimeGenesisConfig {
			system: Default::default(),
			balances: BalancesConfig { balances: self.balances },
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			// Create collections to reserve collection IDs from 0 to 2.
			create_collection(BOB); // reserve collection 0
			create_collection(BOB); // reserve collection 1
			create_collection(BOB); // reserve collection 2

			if let Some(creator) = self.creator {
				Creator::<Test>::put(creator)
			}
			if self.create_contract_collection {
				let creator = Creator::<Test>::get().unwrap();
				let collection_id = create_collection(creator);
				ContractCollectionId::<Test>::put(collection_id);
			}

			// Fund / mint into creator enough to create contracts.
			if let Some((contract_id, contract, should_fund)) = self.contract {
				create_contract(contract_id, contract, should_fund);
			}

			self.stakes.iter().for_each(|(staker, stakes)| {
				stakes.iter().for_each(|(NftId(collection_id, item_id), key, value)| {
					let _ = mint_item(staker, collection_id, item_id);
					set_attribute(collection_id, item_id, key, value);
				})
			});
			self.fees.iter().for_each(|(staker, fees)| {
				fees.iter().for_each(|(NftId(collection_id, item_id), key, value)| {
					let _ = mint_item(staker, collection_id, item_id);
					set_attribute(collection_id, item_id, key, value);
				})
			});
			if let Some((contract_id, who)) = self.accept_contract {
				let stake_addresses = self
					.stakes
					.into_iter()
					.filter(|(staker, _)| staker == &who)
					.flat_map(|(_, stakes)| {
						stakes.into_iter().map(|(address, _, _)| address).collect::<Vec<_>>()
					})
					.collect::<Vec<_>>();
				let fee_addresses = self
					.fees
					.into_iter()
					.filter(|(staker, _)| staker == &who)
					.flat_map(|(_, stakes)| {
						stakes.into_iter().map(|(address, _, _)| address).collect::<Vec<_>>()
					})
					.collect::<Vec<_>>();
				NftStake::accept_contract(contract_id, who, &stake_addresses, &fee_addresses)
					.unwrap();
			}

			if let Some((sniper, contract)) = self.create_sniper {
				let contract_id = H256::random();
				create_contract(contract_id, contract, true);
				NftStake::accept_contract(
					contract_id,
					sniper,
					Default::default(),
					Default::default(),
				)
				.unwrap()
			}
		});
		ext
	}
}

pub fn create_collection(account: MockAccountId) -> MockCollectionId {
	let _ = CurrencyOf::<Test>::deposit_creating(&account, CollectionDeposit::get() + 999);
	let config =
		CollectionConfig { settings: CollectionSettings::all_enabled(), ..Default::default() };
	NftHelperOf::<Test>::create_collection(&account, &account, &config).unwrap()
}

pub fn create_contract(contract_id: MockItemId, contract: ContractOf<Test>, should_fund: bool) {
	let creator = Creator::<Test>::get().unwrap();
	for reward in &contract.rewards {
		match reward {
			Reward::Tokens(amount) =>
				if should_fund {
					let _ =
						CurrencyOf::<Test>::deposit_creating(&creator, ItemDeposit::get() + amount);
				},
			Reward::Nft(NftId(collection_id, item_id)) => {
				let _ = mint_item(&creator, collection_id, item_id);
			},
		}
	}

	if should_fund {
		let _ = CurrencyOf::<Test>::deposit_creating(&creator, ItemDeposit::get());
	}
	NftStake::create_contract(creator, contract_id, contract, None, None).unwrap();
}

pub fn mint_item(
	owner: &MockAccountId,
	collection_id: &MockCollectionId,
	item_id: &MockItemId,
) -> NftIdOf<Test> {
	let _ = CurrencyOf::<Test>::deposit_creating(owner, ItemDeposit::get() + 999);
	let config = pallet_nfts::ItemConfig { settings: ItemSettings::all_enabled() };
	NftHelperOf::<Test>::mint_into(collection_id, item_id, owner, &config, false).unwrap();
	NftId(*collection_id, *item_id)
}

fn set_attribute(
	collection_id: &MockCollectionId,
	item_id: &MockItemId,
	key: &Attribute<KeyLimit>,
	value: &Attribute<ValueLimit>,
) {
	<NftHelperOf<Test> as Mutate<MockAccountId, ItemConfig>>::set_attribute(
		collection_id,
		item_id,
		key.as_slice(),
		value.as_slice(),
	)
	.unwrap()
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			NftStake::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		NftStake::on_initialize(System::block_number());
	}
}
