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
use sp_runtime::{
	testing::{Header, H256},
	traits::{BlakeTwo256, IdentityLookup},
};

pub type MockAccountId = u32;
pub type MockBlockNumber = u64;
pub type MockBalance = u64;
pub type MockIndex = u64;
pub type MockCollectionId = u32;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;
pub const DAVE: MockAccountId = 4;

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
}

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const CollectionDeposit: MockBalance = 1;
	pub const ItemDeposit: MockBalance = 1;
	pub const StringLimit: u32 = 128;
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 200;
	pub const MetadataDepositBase: MockBalance = 1;
	pub const AttributeDepositBase: MockBalance = 1;
	pub const DepositPerByte: MockBalance = 1;
	pub const ApprovalsLimit: u32 = 1;
	pub const ItemAttributesApprovalsLimit: u32 = 10;
	pub const MaxTips: u32 = 1;
	pub const MaxDeadlineDuration: u32 = 1;
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
	type Features = ConfigFeatures;
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
	type AvatarNftConfig = pallet_nfts::ItemConfig;
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
	type NftHelper = Nft;
}

pub struct ExtBuilder {
	existential_deposit: MockBalance,
	organizer: Option<MockAccountId>,
	seasons: Vec<(SeasonId, Season<MockBlockNumber>)>,
	mint_cooldown: Option<MockBlockNumber>,
	mint_fees: Option<MintFees<MockBalance>>,
	trade_min_fee: Option<MockBalance>,
	balances: Vec<(MockAccountId, MockBalance)>,
	free_mints: Vec<(MockAccountId, MintCount)>,
	avatar_transfer_fee: Option<MockBalance>,
	create_nft_collection: bool,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: MockExistentialDeposit::get(),
			organizer: Default::default(),
			seasons: Default::default(),
			mint_cooldown: Default::default(),
			mint_fees: Default::default(),
			trade_min_fee: Default::default(),
			balances: Default::default(),
			free_mints: Default::default(),
			avatar_transfer_fee: Default::default(),
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
	pub fn seasons(mut self, seasons: &[(SeasonId, Season<MockBlockNumber>)]) -> Self {
		self.seasons = seasons.to_vec();
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
	pub fn trade_min_fee(mut self, trade_min_fee: MockBalance) -> Self {
		self.trade_min_fee = Some(trade_min_fee);
		self
	}
	pub fn avatar_transfer_fee(mut self, avatar_transfer_fee: MockBalance) -> Self {
		self.avatar_transfer_fee = Some(avatar_transfer_fee);
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
			});

			if let Some(x) = self.mint_cooldown {
				GlobalConfigs::<Test>::mutate(|config| config.mint.cooldown = x);
			}
			if let Some(x) = self.mint_fees {
				GlobalConfigs::<Test>::mutate(|config| config.mint.fees = x);
			}

			if let Some(x) = self.trade_min_fee {
				GlobalConfigs::<Test>::mutate(|config| config.trade.min_fee = x);
			}

			for (account_id, mint_amount) in self.free_mints {
				Accounts::<Test>::mutate(account_id, |account| account.free_mints = mint_amount);
			}

			if let Some(x) = self.avatar_transfer_fee {
				GlobalConfigs::<Test>::mutate(|config| config.transfer.avatar_transfer_fee = x);
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

impl GlobalConfigOf<Test> {
	pub(crate) fn mint_fees_one(mut self, amount: MockBalance) -> Self {
		self.mint.fees.one = amount;
		self
	}
	pub(crate) fn mint_fees_three(mut self, amount: MockBalance) -> Self {
		self.mint.fees.three = amount;
		self
	}
	pub(crate) fn mint_fees_six(mut self, amount: MockBalance) -> Self {
		self.mint.fees.six = amount;
		self
	}
	pub(crate) fn transfer_avatar_transfer_fee(mut self, amount: MockBalance) -> Self {
		self.transfer.avatar_transfer_fee = amount;
		self
	}
	pub(crate) fn trade_min_fee(mut self, amount: MockBalance) -> Self {
		self.trade.min_fee = amount;
		self
	}
	pub(crate) fn account_storage_upgrade_fe(mut self, amount: MockBalance) -> Self {
		self.account.storage_upgrade_fee = amount;
		self
	}
}
