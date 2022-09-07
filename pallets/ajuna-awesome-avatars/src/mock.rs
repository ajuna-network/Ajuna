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
use frame_support::traits::{ConstU16, ConstU64, Hooks};
use frame_system::mocking::{MockBlock, MockUncheckedExtrinsic};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
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
		AwesomeAvatars: pallet_ajuna_awesome_avatars,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = MockIndex;
	type BlockNumber = MockBlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = MockAccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
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

impl pallet_balances::Config for Test {
	type Balance = MockBalance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = frame_support::traits::ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

impl pallet_ajuna_awesome_avatars::Config for Test {
	type Event = Event;
	type Currency = Balances;
}

#[derive(Default)]
pub struct ExtBuilder {
	organizer: Option<MockAccountId>,
	seasons: Vec<Season<MockBlockNumber>>,
	mint_availability: bool,
	mint_cooldown: Option<MockBlockNumber>,
	balances: Option<Vec<(MockAccountId, MockBalance)>>,
}

impl ExtBuilder {
	pub fn organizer(mut self, organizer: MockAccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}
	pub fn seasons(mut self, seasons: Vec<Season<MockBlockNumber>>) -> Self {
		self.seasons = seasons;
		self
	}
	pub fn mint_availability(mut self, mint_availability: bool) -> Self {
		self.mint_availability = mint_availability;
		self
	}
	pub fn mint_cooldown(mut self, mint_cooldown: MockBlockNumber) -> Self {
		self.mint_cooldown = Some(mint_cooldown);
		self
	}
	pub fn balances(mut self, balances: Vec<(MockAccountId, MockBalance)>) -> Self {
		self.balances = Some(balances);
		self
	}
	pub fn build(self) -> sp_io::TestExternalities {
		let balances = self.balances.unwrap_or(Default::default());
		let config =
			GenesisConfig { system: Default::default(), balances: BalancesConfig { balances } };

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			if let Some(organizer) = self.organizer {
				Organizer::<Test>::put(organizer);
			}

			for season in self.seasons {
				let season_id = AwesomeAvatars::next_season_id();
				Seasons::<Test>::insert(season_id, season);
				NextSeasonId::<Test>::put(season_id + 1);
			}

			MintAvailable::<Test>::set(self.mint_availability);

			if let Some(mint_cooldown) = self.mint_cooldown {
				MintCooldown::<Test>::set(mint_cooldown);
			}
		});
		ext
	}
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			AwesomeAvatars::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		AwesomeAvatars::on_initialize(System::block_number());
	}
}

pub fn test_rarity_tiers(rarity_tiers: Vec<(RarityTier, RarityPercent)>) -> RarityTiers {
	rarity_tiers.try_into().unwrap()
}

impl Default for Season<MockBlockNumber> {
	fn default() -> Self {
		Self {
			early_start: 1,
			start: 2,
			end: 3,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: test_rarity_tiers(vec![
				(RarityTier::Common, 50),
				(RarityTier::Uncommon, 30),
				(RarityTier::Rare, 12),
				(RarityTier::Epic, 5),
				(RarityTier::Legendary, 2),
				(RarityTier::Mythical, 1),
			]),
			max_variations: 1,
			max_components: 1,
		}
	}
}

impl Season<MockBlockNumber> {
	pub fn early_start(mut self, early_start: MockBlockNumber) -> Self {
		self.early_start = early_start;
		self
	}
	pub fn start(mut self, start: MockBlockNumber) -> Self {
		self.start = start;
		self
	}
	pub fn end(mut self, end: MockBlockNumber) -> Self {
		self.end = end;
		self
	}
	pub fn rarity_tiers(mut self, rarity_tiers: RarityTiers) -> Self {
		self.rarity_tiers = rarity_tiers;
		self
	}
	pub fn max_components(mut self, max_components: u8) -> Self {
		self.max_components = max_components;
		self
	}
}
