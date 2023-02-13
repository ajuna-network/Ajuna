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

use crate::{self as pallet_ajuna_awesome_avatars, types::*, *};
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU64, GenesisBuild, Hooks},
};
use frame_system::{
	mocking::{MockBlock, MockUncheckedExtrinsic},
	EnsureRoot, EnsureSigned,
};
use sp_runtime::{
	testing::{Header, H256},
	traits::{BlakeTwo256, IdentityLookup},
};

pub type MockAccountId = u32;
pub type MockBlockNumber = u64;
pub type MockBalance = u64;
pub type MockIndex = u64;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;
pub const DELTHEA: MockAccountId = 4;
pub const ERIN: MockAccountId = 5;
pub const FLORINA: MockAccountId = 6;
pub const HILDA: MockAccountId = 7;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = MockBlock<Test>,
		NodeBlock = MockBlock<Test>,
		UncheckedExtrinsic = MockUncheckedExtrinsic<Test>,
	{
		System: frame_system,
		Balances: pallet_balances,
		Randomness: pallet_randomness_collective_flip,
		Nft: pallet_nfts,
		AAvatars: pallet_ajuna_awesome_avatars,
		NftTransfer: pallet_ajuna_nft_transfer,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
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

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const CollectionDeposit: MockBalance = 1;
	pub const ItemDeposit: MockBalance = 1;
	pub const StringLimit: u32 = 128;
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 64;
	pub const MetadataDepositBase: MockBalance = 1;
	pub const AttributeDepositBase: MockBalance = 1;
	pub const DepositPerByte: MockBalance = 1;
	pub const ApprovalsLimit: u32 = 1;
	pub const ItemAttributesApprovalsLimit: u32 = 10;
	pub const MaxTips: u32 = 1;
	pub const MaxDeadlineDuration: u32 = 1;
	pub ConfigFeatures: pallet_nfts::PalletFeatures = pallet_nfts::PalletFeatures::all_enabled();
}

pub type MockCollectionId = u32;
pub type MockItemId = u128;

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
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const AvatarCollection: MockCollectionId = 0;
}

impl pallet_ajuna_awesome_avatars::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Randomness = Randomness;
	type AvatarNftHandler = NftTransfer;
	type AvatarCollectionId = MockCollectionId;
	type AvatarCollection = AvatarCollection;
	type AvatarItemId = MockItemId;
	type AvatarItemConfig = pallet_nfts::ItemConfig;
	type WeightInfo = ();
}

pub const MAX_ENCODING_SIZE: u32 = 200;

pub type CollectionConfig =
	pallet_nfts::CollectionConfig<MockBalance, MockBlockNumber, MockCollectionId>;

impl pallet_ajuna_nft_transfer::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxAssetEncodedSize = frame_support::traits::ConstU32<MAX_ENCODING_SIZE>;
	type CollectionId = MockCollectionId;
	type CollectionConfig = CollectionConfig;
	type ItemId = MockItemId;
	type ItemConfig = pallet_nfts::ItemConfig;
	type NftHelper = Nft;
}

pub struct ExtBuilder {
	organizer: Option<MockAccountId>,
	seasons: Vec<(SeasonId, Season<MockBlockNumber>)>,
	mint_open: bool,
	mint_cooldown: Option<MockBlockNumber>,
	mint_fees: Option<MintFees<MockBalance>>,
	forge_open: bool,
	trade_open: bool,
	trade_min_fee: Option<MockBalance>,
	balances: Vec<(MockAccountId, MockBalance)>,
	free_mints: Vec<(MockAccountId, MintCount)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			organizer: Default::default(),
			seasons: Default::default(),
			mint_cooldown: Default::default(),
			mint_fees: Default::default(),
			balances: Default::default(),
			free_mints: Default::default(),
			mint_open: true,
			forge_open: true,
			trade_open: true,
			trade_min_fee: Default::default(),
		}
	}
}

impl ExtBuilder {
	pub fn organizer(mut self, organizer: MockAccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}
	pub fn seasons(mut self, seasons: &[(SeasonId, Season<MockBlockNumber>)]) -> Self {
		self.seasons = seasons.to_vec();
		self
	}
	pub fn mint_open(mut self, mint_open: bool) -> Self {
		self.mint_open = mint_open;
		self
	}
	pub fn mint_cooldown(mut self, mint_cooldown: MockBlockNumber) -> Self {
		self.mint_cooldown = Some(mint_cooldown);
		self
	}
	pub fn balances(mut self, balances: &[(MockAccountId, MockBalance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}
	pub fn mint_fees(mut self, mint_fees: MintFees<MockBalance>) -> Self {
		self.mint_fees = Some(mint_fees);
		self
	}
	pub fn free_mints(mut self, free_mints: &[(MockAccountId, MintCount)]) -> Self {
		self.free_mints = free_mints.to_vec();
		self
	}
	pub fn forge_open(mut self, forge_open: bool) -> Self {
		self.forge_open = forge_open;
		self
	}
	pub fn trade_open(mut self, trade_open: bool) -> Self {
		self.trade_open = trade_open;
		self
	}
	pub fn trade_min_fee(mut self, trade_min_fee: MockBalance) -> Self {
		self.trade_min_fee = Some(trade_min_fee);
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();
		pallet_ajuna_awesome_avatars::GenesisConfig::<Test>::default()
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			if let Some(organizer) = self.organizer {
				Organizer::<Test>::put(organizer);
			}

			for (season_id, season) in self.seasons {
				Seasons::<Test>::insert(season_id, season);
			}

			GlobalConfigs::<Test>::mutate(|config| config.mint.open = self.mint_open);
			if let Some(x) = self.mint_cooldown {
				GlobalConfigs::<Test>::mutate(|config| config.mint.cooldown = x);
			}
			if let Some(x) = self.mint_fees {
				GlobalConfigs::<Test>::mutate(|config| config.mint.fees = x);
			}

			GlobalConfigs::<Test>::mutate(|config| {
				config.forge.open = self.forge_open;
				config.trade.open = self.trade_open;
			});

			if let Some(x) = self.trade_min_fee {
				GlobalConfigs::<Test>::mutate(|config| config.trade.min_fee = x);
			}

			for (account_id, mint_amount) in self.free_mints {
				Accounts::<Test>::mutate(account_id, |account| account.free_mints = mint_amount);
			}

			Nft::force_create(
				RuntimeOrigin::root(),
				ALICE,
				pallet_nfts::CollectionConfig::default(),
			)
			.expect("Collection created");
		});
		ext
	}
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			AAvatars::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		AAvatars::on_initialize(System::block_number());
	}
}
