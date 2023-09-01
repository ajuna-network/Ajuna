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
	traits::{
		tokens::nonfungibles_v2::Create, AsEnsureOriginWithArg, ConstU16, ConstU64, GenesisBuild,
		Hooks,
	},
	PalletId,
};
use frame_system::{
	mocking::{MockBlock, MockUncheckedExtrinsic},
	EnsureRoot, EnsureSigned,
};
pub(crate) use sp_runtime::testing::H256;
use sp_runtime::{
	testing::{Header, TestSignature},
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
};

pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlockNumber = u64;
pub type MockBalance = u64;
pub type MockIndex = u64;
pub type MockCollectionId = u32;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;
pub const DAVE: MockAccountId = 4;

pub const SEASON_ID: SeasonId = 1;

frame_support::construct_runtime!(
	pub enum Test where
		Block = MockBlock<Test>,
		NodeBlock = MockBlock<Test>,
		UncheckedExtrinsic = MockUncheckedExtrinsic<Test>,
	{
		System: frame_system,
		Balances: pallet_balances,
		Randomness: pallet_insecure_randomness_collective_flip,
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
	pub static MockExistentialDeposit: MockBalance = 321;
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
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const CollectionDeposit: MockBalance = 1;
	pub const ItemDeposit: MockBalance = 1;
	pub const StringLimit: u32 = 128;
	pub const MetadataDepositBase: MockBalance = 1;
	pub const AttributeDepositBase: MockBalance = 1;
	pub const DepositPerByte: MockBalance = 1;
	pub const ApprovalsLimit: u32 = 1;
	pub const ItemAttributesApprovalsLimit: u32 = 10;
	pub const MaxTips: u32 = 1;
	pub const MaxDeadlineDuration: u32 = 1;
	pub const MaxAttributesPerCall: u32 = 10;
	pub ConfigFeatures: pallet_nfts::PalletFeatures = pallet_nfts::PalletFeatures::all_enabled();
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
	type ItemId = H256;
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
	type MaxAttributesPerCall = MaxAttributesPerCall;
	type Features = ConfigFeatures;
	type OffchainSignature = MockSignature;
	type OffchainPublic = MockAccountPublic;
	pallet_nfts::runtime_benchmarks_enabled! {
		type Helper = Helper;
	}
	type WeightInfo = ();
}

parameter_types! {
	pub const AwesomeAvatarsPalletId: PalletId = PalletId(*b"aj/aaatr");
}

impl pallet_ajuna_awesome_avatars::Config for Test {
	type PalletId = AwesomeAvatarsPalletId;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Randomness = Randomness;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type NftHandler = NftTransfer;
	type WeightInfo = ();
}

parameter_types! {
	pub const NftTransferPalletId: PalletId = PalletId(*b"aj/nfttr");
}

impl pallet_ajuna_nft_transfer::Config for Test {
	type PalletId = NftTransferPalletId;
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = MockCollectionId;
	type ItemId = H256;
	type ItemConfig = pallet_nfts::ItemConfig;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type NftHelper = Nft;
}

pub struct ExtBuilder {
	existential_deposit: MockBalance,
	organizer: Option<MockAccountId>,
	seasons: Vec<(SeasonId, Season<MockBlockNumber, MockBalance>)>,
	mint_cooldown: MockBlockNumber,
	balances: Vec<(MockAccountId, MockBalance)>,
	free_mints: Vec<(MockAccountId, MintCount)>,
	create_nft_collection: bool,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: MockExistentialDeposit::get(),
			organizer: Default::default(),
			seasons: Default::default(),
			mint_cooldown: Default::default(),
			balances: Default::default(),
			free_mints: Default::default(),
			create_nft_collection: Default::default(),
		}
	}
}

impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: MockBalance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}
	pub fn organizer(mut self, organizer: MockAccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}
	pub fn seasons(mut self, seasons: &[(SeasonId, Season<MockBlockNumber, MockBalance>)]) -> Self {
		self.seasons = seasons.to_vec();
		self
	}
	pub fn mint_cooldown(mut self, mint_cooldown: MockBlockNumber) -> Self {
		self.mint_cooldown = mint_cooldown;
		self
	}
	pub fn balances(mut self, balances: &[(MockAccountId, MockBalance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}
	pub fn free_mints(mut self, free_mints: &[(MockAccountId, MintCount)]) -> Self {
		self.free_mints = free_mints.to_vec();
		self
	}
	pub fn create_nft_collection(mut self, create_nft_collection: bool) -> Self {
		self.create_nft_collection = create_nft_collection;
		self
	}
	pub fn build(self) -> sp_io::TestExternalities {
		MOCK_EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
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

			GlobalConfigs::<Test>::mutate(|config| {
				config.mint.open = true;
				config.forge.open = true;
				config.trade.open = true;
				config.mint.cooldown = self.mint_cooldown;
			});

			for (account_id, mint_amount) in self.free_mints {
				PlayerConfigs::<Test>::mutate(account_id, |account| {
					account.free_mints = mint_amount
				});
			}

			if self.create_nft_collection {
				let collection_id = Nft::create_collection(
					&ALICE,
					&ALICE,
					&pallet_nfts::CollectionConfig::default(),
				)
				.expect("Collection created");
				CollectionId::<Test>::put(collection_id);
			}
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
