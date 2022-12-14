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

use crate::{self as pallet_ajuna_board};
use frame_support::parameter_types;
use frame_system::mocking::{MockBlock, MockUncheckedExtrinsic};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};
use sp_std::prelude::*;

type MockAccountId = u32;

frame_support::construct_runtime!(
	pub enum Test where
		Block = MockBlock<Test>,
		NodeBlock = MockBlock<Test>,
		UncheckedExtrinsic = MockUncheckedExtrinsic<Test>,
	{
		System: frame_system,
		AjunaMatchmaker: pallet_ajuna_matchmaker,
		AjunaBoard: pallet_ajuna_board,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = MockAccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_ajuna_matchmaker::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub const Players: u8 = 2;
}

impl pallet_ajuna_board::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Matchmaker = pallet_ajuna_matchmaker::Matchmaking<Self>;
	type BoardId = u32;
	type PlayersTurn = crate::dot4gravity::Turn;
	type GameState = crate::dot4gravity::GameState<MockAccountId>;
	type Game = crate::dot4gravity::Game<MockAccountId>;
	type Players = Players;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let config = GenesisConfig { system: Default::default() };
	let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
