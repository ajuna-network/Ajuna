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

use crate::{self as pallet_ajuna_awesome_avatars, types::Season};
use frame_support::traits::{ConstU16, ConstU64};
use frame_system::mocking::{MockBlock, MockUncheckedExtrinsic};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type MockAccountId = u32;
type MockBlockNumber = u64;
type MockBalance = u64;

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
	type Index = u64;
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
	pub fn build(self) -> sp_io::TestExternalities {
		let config = GenesisConfig { system: Default::default(), balances: Default::default() };
		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			if let Some(organizer) = self.organizer {
				let _ = AwesomeAvatars::set_organizer(Origin::root(), organizer);
			}

			for season in self.seasons.into_iter() {
				let organizer = AwesomeAvatars::organizer().unwrap();
				let _ = AwesomeAvatars::new_season(Origin::signed(organizer), season);
			}
		});
		ext
	}
}
