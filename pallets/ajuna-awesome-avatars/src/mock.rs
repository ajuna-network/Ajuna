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
		AAvatars: pallet_ajuna_awesome_avatars,
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

impl pallet_randomness_collective_flip::Config for Test {}

impl pallet_ajuna_awesome_avatars::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type Randomness = Randomness;
}

#[derive(Default)]
pub struct ExtBuilder {
	organizer: Option<MockAccountId>,
	seasons: Vec<(SeasonId, Season<MockBlockNumber>)>,
	max_avatars_per_player: Option<u32>,
	mint_open: bool,
	mint_cooldown: Option<MockBlockNumber>,
	mint_fees: Option<MintFees<MockBalance>>,
	forge_open: bool,
	forge_min_sacrifices: Option<u8>,
	forge_max_sacrifices: Option<u8>,
	trade_open: bool,
	balances: Vec<(MockAccountId, MockBalance)>,
	free_mints: Vec<(MockAccountId, MintCount)>,
}

impl ExtBuilder {
	pub fn organizer(mut self, organizer: MockAccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}
	pub fn seasons(mut self, seasons: Vec<(SeasonId, Season<MockBlockNumber>)>) -> Self {
		self.seasons = seasons;
		self
	}
	pub fn max_avatars_per_player(mut self, max_avatars_per_player: u32) -> Self {
		self.max_avatars_per_player = Some(max_avatars_per_player);
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
	pub fn balances(mut self, balances: Vec<(MockAccountId, MockBalance)>) -> Self {
		self.balances = balances;
		self
	}
	pub fn mint_fees(mut self, mint_fees: MintFees<MockBalance>) -> Self {
		self.mint_fees = Some(mint_fees);
		self
	}
	pub fn free_mints(mut self, free_mints: Vec<(MockAccountId, MintCount)>) -> Self {
		self.free_mints = free_mints;
		self
	}
	pub fn forge_open(mut self, forge_open: bool) -> Self {
		self.forge_open = forge_open;
		self
	}
	pub fn forge_min_sacrifices(mut self, forge_min_sacrifices: u8) -> Self {
		self.forge_min_sacrifices = Some(forge_min_sacrifices);
		self
	}
	pub fn forge_max_sacrifices(mut self, forge_max_sacrifices: u8) -> Self {
		self.forge_max_sacrifices = Some(forge_max_sacrifices);
		self
	}
	pub fn trade_open(mut self, trade_open: bool) -> Self {
		self.trade_open = trade_open;
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
			if let Some(organizer) = self.organizer {
				Organizer::<Test>::put(organizer);
			}

			for (season_id, season) in self.seasons {
				Seasons::<Test>::insert(season_id, season);
			}

			if let Some(x) = self.max_avatars_per_player {
				GlobalConfigs::<Test>::mutate(|config| config.max_avatars_per_player = x);
			}

			GlobalConfigs::<Test>::mutate(|config| config.mint.open = self.mint_open);
			if let Some(x) = self.mint_cooldown {
				GlobalConfigs::<Test>::mutate(|config| config.mint.cooldown = x);
			}
			if let Some(x) = self.mint_fees {
				GlobalConfigs::<Test>::mutate(|config| config.mint.fees = x);
			}

			GlobalConfigs::<Test>::mutate(|config| config.forge.open = self.forge_open);
			if let Some(x) = self.forge_min_sacrifices {
				GlobalConfigs::<Test>::mutate(|config| config.forge.min_sacrifices = x);
			}
			if let Some(x) = self.forge_max_sacrifices {
				GlobalConfigs::<Test>::mutate(|config| config.forge.max_sacrifices = x);
			}

			GlobalConfigs::<Test>::mutate(|config| config.trade.open = self.trade_open);
			for (account_id, mint_amount) in self.free_mints {
				FreeMints::<Test>::insert(account_id, mint_amount);
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

impl Default for Season<MockBlockNumber> {
	fn default() -> Self {
		Self {
			name: b"cool season".to_vec().try_into().unwrap(),
			description: b"this is a really cool season".to_vec().try_into().unwrap(),
			early_start: 1,
			start: 2,
			end: 3,
			max_variations: 2,
			max_components: 2,
			tiers: vec![
				RarityTier::Common,
				RarityTier::Uncommon,
				RarityTier::Rare,
				RarityTier::Epic,
				RarityTier::Legendary,
				RarityTier::Mythical,
			]
			.try_into()
			.unwrap(),
			p_single_mint: vec![50, 30, 15, 4, 1].try_into().unwrap(),
			p_batch_mint: vec![50, 30, 15, 4, 1].try_into().unwrap(),
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
	pub fn max_components(mut self, max_components: u8) -> Self {
		self.max_components = max_components;
		self
	}
	pub fn max_variations(mut self, max_variations: u8) -> Self {
		self.max_variations = max_variations;
		self
	}
	pub fn tiers(mut self, tiers: Vec<RarityTier>) -> Self {
		self.tiers = tiers.try_into().unwrap();
		self
	}
	pub fn p_single_mint(mut self, percentages: Vec<RarityPercent>) -> Self {
		self.p_single_mint = percentages.try_into().unwrap();
		self
	}
	pub fn p_batch_mint(mut self, percentages: Vec<RarityPercent>) -> Self {
		self.p_batch_mint = percentages.try_into().unwrap();
		self
	}
}
