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
	testing::{Header, H256},
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

pub type MockAccountId = u32;
pub type MockBlockNumber = u64;
pub type MockBalance = u64;
pub type MockIndex = u64;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;

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
	pub const MockExistentialDeposit: MockBalance = 321;
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
	pub const CollectionDeposit: MockBalance = 0;
	pub const ItemDeposit: MockBalance = 0;
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
	type Features = ConfigFeatures;
	pallet_nfts::runtime_benchmarks_enabled! {
		type Helper = ();
	}
	type WeightInfo = ();
}

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"aj/nftst");
	pub const MinimumStakingTokenReward: MockBalance = 100;
	pub ContractCollectionConfig: CollectionConfig = CollectionConfig::default();
	pub ContractCollectionItemConfig: pallet_nfts::ItemConfig = pallet_nfts::ItemConfig::default();
}

pub type CollectionConfig =
	pallet_nfts::CollectionConfig<MockBalance, MockBlockNumber, MockCollectionId>;

pub type ContractAttributeKey = u32;
pub type ContractAttributeValue = u64;

impl pallet_nft_staking::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CollectionId = MockCollectionId;
	type CollectionConfig = CollectionConfig;
	type ItemId = MockItemId;
	type ItemConfig = pallet_nfts::ItemConfig;
	type NftHelper = Nft;
	type StakingOrigin = EnsureSigned<MockAccountId>;
	type TreasuryPalletId = TreasuryPalletId;
	type MinimumStakingTokenReward = MinimumStakingTokenReward;
	type ContractCollectionConfig = ContractCollectionConfig;
	type ContractCollectionItemConfig = ContractCollectionItemConfig;
	type ContractAttributeKey = ContractAttributeKey;
	type ContractAttributeValue = ContractAttributeValue;
	type WeightInfo = ();
}

pub struct ExtBuilder {
	balances: Vec<(MockAccountId, MockBalance)>,
	create_collection: bool,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		let accounts_balance: MockBalance = 1_000_000_000_000;

		Self {
			balances: vec![
				(ALICE, accounts_balance),
				(BOB, accounts_balance),
				(CHARLIE, accounts_balance),
			],
			create_collection: true,
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, balances: Vec<(MockAccountId, MockBalance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn create_collection(mut self, create_collection: bool) -> Self {
		self.create_collection = create_collection;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let config = GenesisConfig {
			system: Default::default(),
			balances: BalancesConfig { balances: self.balances },
			nft_stake: Default::default(),
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		if self.create_collection {
			ext.execute_with(|| {
				let account_id = <Pallet<Test>>::treasury_account_id();
				let collection_config =
					<Test as crate::pallet::Config>::ContractCollectionConfig::get();
				let collection_id = <Test as crate::pallet::Config>::NftHelper::create_collection(
					&account_id,
					&account_id,
					&collection_config,
				)
				.expect("Should have create contract collection");
				ContractCollectionId::<Test>::put(collection_id);
			});
		}
		ext
	}
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
