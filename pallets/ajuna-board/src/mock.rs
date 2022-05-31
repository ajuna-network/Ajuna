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

use crate::{self as pallet_ajuna_board, TurnBasedGame};
use frame_support::parameter_types;
use frame_system::mocking::{MockBlock, MockUncheckedExtrinsic};
use sp_core::{Decode, Encode, H256};
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
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		AjunaBoard: pallet_ajuna_board::{Pallet, Call, Storage, Event<T>},
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
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = MockAccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
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

parameter_types! {
	pub const MaxNumberOfPlayers: u8 = 2;
}

pub type Guess = u32;

use frame_support::{pallet_prelude::MaxEncodedLen, RuntimeDebugNoBound};
use scale_info::TypeInfo;

const MAX_PLAYERS: usize = 2;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
pub struct GameState {
	pub players: [MockAccountId; MAX_PLAYERS],
	pub next_player: u8,
	pub solution: Guess,
	pub winner: Option<MockAccountId>,
}

// Rules
// One player can only have one go at a time
// It's a guessing game where a player has to guess the right number
// Initial state will have this number

pub const THE_NUMBER: Guess = 42;

pub struct MockGame;

impl TurnBasedGame for MockGame {
	type State = GameState;
	type Player = MockAccountId;
	type Turn = Guess;

	fn init(players: &[Self::Player]) -> Option<Self::State> {
		if players.len() != MAX_PLAYERS {
			return None
		};

		let mut p: [Self::Player; MAX_PLAYERS] = Default::default();
		p.copy_from_slice(&players[0..MAX_PLAYERS]);
		Some(GameState { players: p, next_player: 0, solution: THE_NUMBER, winner: None })
	}

	fn play_turn(
		player: Self::Player,
		state: Self::State,
		turn: Self::Turn,
	) -> Option<Self::State> {
		if state.winner.is_some() {
			return None
		}

		if !state.players.contains(&player) {
			return None
		}

		if state.players[state.next_player as usize] != player {
			return None
		}

		let mut state = state;
		state.next_player = (state.next_player + 1) % state.players.len() as u8;

		if state.solution == turn {
			state.winner = Some(player);
		}

		Some(state)
	}

	fn is_finished(state: &Self::State) -> pallet_ajuna_board::Finished<Self::Player> {
		match state.winner {
			None => pallet_ajuna_board::Finished::No,
			Some(winner) => pallet_ajuna_board::Finished::Winner(winner),
		}
	}
}

impl pallet_ajuna_board::Config for Test {
	type Event = Event;
	type MaxNumberOfPlayers = MaxNumberOfPlayers;
	type BoardId = u32;
	type PlayersTurn = u32;
	type GameState = GameState;
	type Game = MockGame;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let config = GenesisConfig { system: Default::default() };

	let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();

	ext.execute_with(|| {
		System::set_block_number(1);
	});

	ext
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Test>::events().pop().expect("Event expected").event
}
