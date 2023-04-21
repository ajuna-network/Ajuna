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
use frame_system::{
	mocking::{MockBlock, MockUncheckedExtrinsic},
	EnsureRoot, EnsureSigned,
};
use pallet_nfts::PalletFeatures;
use sp_runtime::{
	testing::{Header, TestSignature, H256},
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};

pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlockNumber = u64;
pub type MockBalance = u64;
pub type MockIndex = u64;

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
	pub enum Test where
		Block = MockBlock<Test>,
		NodeBlock = MockBlock<Test>,
		UncheckedExtrinsic = MockUncheckedExtrinsic<Test>,
	{
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
	type Index = MockIndex;
	type BlockNumber = MockBlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = MockAccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
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
}

pub type MockCollectionId = u32;
pub type MockItemId = u128;

parameter_types! {
	pub const CollectionDeposit: MockBalance = 333;
	pub const ItemDeposit: MockBalance = 33;
	pub const MetadataDepositBase: MockBalance = 0;
	pub const AttributeDepositBase: MockBalance = 0;
	pub const DepositPerByte: MockBalance = 0;
	pub const StringLimit: u32 = 128;
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 64;
	pub const ApprovalsLimit: u32 = 1;
	pub const ItemAttributesApprovalsLimit: u32 = 10;
	pub const MaxTips: u32 = 1;
	pub const MaxDeadlineDuration: u32 = 1;
	pub ConfigFeatures: PalletFeatures = PalletFeatures::all_enabled();
}

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
		type Helper = ();
	}
	type WeightInfo = ();
}

parameter_types! {
	pub const NftStakingPalletId: PalletId = PalletId(*b"aj/nftst");
	pub const MaxClauses: u32 = 10;
	pub ContractCollectionItemConfig: pallet_nfts::ItemConfig = pallet_nfts::ItemConfig::default();
}

pub type CollectionConfig =
	pallet_nfts::CollectionConfig<MockBalance, MockBlockNumber, MockCollectionId>;

pub type AttributeKey = u32;
pub type AttributeValue = u64;

impl pallet_nft_staking::Config for Test {
	type PalletId = NftStakingPalletId;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CollectionId = MockCollectionId;
	type CollectionConfig = CollectionConfig;
	type ItemId = MockItemId;
	type ItemConfig = pallet_nfts::ItemConfig;
	type NftHelper = Nft;
	type MaxClauses = MaxClauses;
	type ContractCollectionItemConfig = ContractCollectionItemConfig;
	type ContractAttributeKey = AttributeKey;
	type ContractAttributeValue = AttributeValue;
	type WeightInfo = ();
}

pub type MockClause = Clause<MockCollectionId, AttributeKey, AttributeValue>;
pub struct MockClauses(pub Vec<MockClause>);

#[derive(Clone)]
pub struct MockStakes(
	pub Vec<(NftAddress<MockCollectionId, MockItemId>, AttributeKey, AttributeValue)>,
);

impl From<MockClauses> for MockStakes {
	fn from(clauses: MockClauses) -> Self {
		Self(
			clauses
				.0
				.into_iter()
				.enumerate()
				.map(|(i, clause)| match clause {
					Clause::HasAttribute(collection_id, key) =>
						(NftAddress(collection_id, i as MockItemId), key, (i + 1) as AttributeValue),
					Clause::HasAttributeWithValue(collection_id, key, value) =>
						(NftAddress(collection_id, i as MockItemId), key, value),
				})
				.collect(),
		)
	}
}

impl MockStakes {
	// Increment item ID to ensure unique items are minted.
	pub fn inc_item_id(self) -> Self {
		Self(
			self.0
				.into_iter()
				.map(|(NftAddress(collection_id, item_id), key, value)| {
					(NftAddress(collection_id, item_id + 1), key, value)
				})
				.collect(),
		)
	}
}

#[derive(Default)]
pub struct ExtBuilder {
	creator: Option<MockAccountId>,
	balances: Vec<(MockAccountId, MockBalance)>,
	create_contract_collection: bool,
	contracts: Vec<ContractOf<Test>>,
	stakes: Vec<(MockAccountId, MockStakes)>,
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
	pub fn contracts(mut self, contracts: Vec<ContractOf<Test>>) -> Self {
		self.contracts = contracts;
		self
	}
	pub fn stakes(mut self, stakes: Vec<(MockAccountId, MockStakes)>) -> Self {
		self.stakes = stakes;
		self
	}
	pub fn build(self) -> sp_io::TestExternalities {
		let config = GenesisConfig {
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

			self.contracts.iter().for_each(|contract| {
				let creator = Creator::<Test>::get().unwrap();

				// Fund creator enough to create contracts.
				match &contract.reward {
					Reward::Tokens(amount) => {
						let _ = CurrencyOf::<Test>::deposit_creating(
							&creator,
							ItemDeposit::get() + amount,
						);
					},
					Reward::Nft(address) => {},
				}
				NftStake::create_contract(&creator, contract).unwrap();
			});

			self.stakes.into_iter().for_each(|(staker, stakes)| {
				stakes
					.0
					.into_iter()
					.for_each(|(NftAddress(collection_id, item_id), key, value)| {
						let _ = mint_item(staker, collection_id, item_id);
						set_attribute(collection_id, item_id, key, value);
					})
			});
		});
		ext
	}
}

pub fn create_collection(account: MockAccountId) -> MockCollectionId {
	let _ = CurrencyOf::<Test>::deposit_creating(&account, CollectionDeposit::get());
	let config = CollectionConfig::default();
	NftHelperOf::<Test>::create_collection(&account, &account, &config).unwrap()
}

pub fn mint_item(
	owner: MockAccountId,
	collection_id: MockCollectionId,
	item_id: MockItemId,
) -> NftAddressOf<Test> {
	let _ = CurrencyOf::<Test>::deposit_creating(&owner, ItemDeposit::get());
	let config = pallet_nfts::ItemConfig::default();
	NftHelperOf::<Test>::mint_into(&collection_id, &item_id, &owner, &config, false).unwrap();
	NftAddress(collection_id, item_id)
}

fn set_attribute(
	collection_id: MockCollectionId,
	item_id: MockItemId,
	key: AttributeKey,
	value: AttributeValue,
) {
	NftHelperOf::<Test>::set_typed_attribute(&collection_id, &item_id, &key, &value).unwrap()
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
