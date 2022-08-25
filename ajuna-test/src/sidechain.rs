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

use crate::impl_block_numbers;
use ajuna_common::{Finished, TurnBasedGame};
use ajuna_solo_runtime::{AccountId, Balance};
use frame_support::parameter_types;
use frame_system::mocking::{MockBlock, MockUncheckedExtrinsic};
use sp_core::{Decode, Encode, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::prelude::*;
use std::marker::PhantomData;

frame_support::construct_runtime!(
	pub enum SideChainRuntime where
		Block = MockBlock<SideChainRuntime>,
		NodeBlock = MockBlock<SideChainRuntime>,
		UncheckedExtrinsic = MockUncheckedExtrinsic<SideChainRuntime>,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		AjunaBoard: pallet_ajuna_board::{Pallet, Call, Storage, Event<T>},
		Vesting: orml_vesting,
		Balances: pallet_balances,
	}
);

parameter_types! {
	pub const ExistentialDeposit: Balance = 100 * NANO_AJUNS;
	pub const ArbitraryUpperBound: u32 = 1_000_000;
}

pub const EXISTENTIAL_DEPOSIT: Balance = 100 * NANO_AJUNS;

impl pallet_balances::Config for SideChainRuntime {
	type MaxLocks = ArbitraryUpperBound;
	type MaxReserves = ArbitraryUpperBound;
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * MICRO_AJUNS;
	pub const MaxVestingSchedules: u32 = 100;
}

impl orml_vesting::Config for SideChainRuntime {
	type Event = Event;
	type Currency = Balances;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = EnsureSigned<AccountId>;
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

type BlockNumber = u64;

impl frame_system::Config for SideChainRuntime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
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

use crate::traits::BlockProcessing;
use ajuna_solo_runtime::currency::{MICRO_AJUNS, NANO_AJUNS};
use frame_support::{pallet_prelude::MaxEncodedLen, RuntimeDebugNoBound};
use frame_system::EnsureSigned;
use pallet_ajuna_board::{dot4gravity, dot4gravity::Turn};
use scale_info::TypeInfo;

const MAX_PLAYERS: usize = 2;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
pub struct GameState {
	pub players: [AccountId; MAX_PLAYERS],
	pub next_player: u8,
	pub solution: Guess,
	pub winner: Option<AccountId>,
	pub seed: u32,
}

// Rules
// One player can only have one go at a time
// It's a guessing game where a player has to guess the right number
// Initial state will have this number

pub const THE_NUMBER: Guess = 42;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
pub struct PlayerTurn(pub Guess);

// We need this to satisfy the current requirements for pallet_ajuna_board::Config
impl From<dot4gravity::Turn> for PlayerTurn {
	fn from(_: Turn) -> Self {
		PlayerTurn(0)
	}
}

pub struct NumberGame;

impl TurnBasedGame for NumberGame {
	type Turn = PlayerTurn;
	type Player = AccountId;
	type State = GameState;

	fn init(players: &[Self::Player], seed: Option<u32>) -> Option<Self::State> {
		if players.len() != MAX_PLAYERS {
			return None
		};

		let mut p = vec![[0u8; 32].into(); MAX_PLAYERS];
		p.clone_from_slice(players);
		Some(GameState {
			players: <[AccountId; MAX_PLAYERS]>::try_from(p).unwrap(),
			next_player: 0,
			solution: THE_NUMBER,
			winner: None,
			seed: seed.unwrap_or_default(),
		})
	}

	fn get_last_player(state: &Self::State) -> Self::Player {
		state.players[(state.next_player as usize - 1) % state.players.len()].clone()
	}

	fn get_next_player(state: &Self::State) -> Self::Player {
		state.players[state.next_player as usize].clone()
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

		if state.solution == turn.0 {
			state.winner = Some(player);
		}

		Some(state)
	}

	fn abort(state: Self::State, winner: Self::Player) -> Self::State {
		let mut state = state;
		state.winner = Some(winner);
		state
	}

	fn is_finished(state: &Self::State) -> Finished<Self::Player> {
		match &state.winner {
			None => Finished::No,
			Some(winner) => Finished::Winner(winner.clone()),
		}
	}

	fn seed(state: &Self::State) -> Option<u32> {
		Some(state.seed)
	}
}

impl pallet_ajuna_board::Config for SideChainRuntime {
	type Event = Event;
	type BoardId = u32;
	type PlayersTurn = PlayerTurn;
	type GameState = GameState;
	type Game = NumberGame;
	type MaxNumberOfPlayers = MaxNumberOfPlayers;
	type WeightInfo = ();
}

pub struct SideChain<K: SigningKey> {
	_k: PhantomData<K>,
}

pub trait SigningKey {
	fn account() -> AccountId;
}

impl_block_numbers!(System, BlockNumber);
impl<K: SigningKey> BlockProcessing<BlockNumber, RuntimeBlocks> for SideChain<K> {
	fn on_block() {
		// Trigger block importer
		block_importer::consume_block(K::account());
	}
}

#[cfg(test)]
impl<K: SigningKey> SideChain<K> {
	// Build genesis storage according to the mock runtime.
	pub fn build() -> sp_io::TestExternalities {
		use crate::keyring::*;
		use sp_runtime::BuildStorage;

		let config = GenesisConfig {
			system: Default::default(),
			balances: pallet_balances::GenesisConfig {
				balances: vec![
					(alice(), 10_000 * EXISTENTIAL_DEPOSIT),
					(bob(), 20_000 * EXISTENTIAL_DEPOSIT),
					(charlie(), 30_000 * EXISTENTIAL_DEPOSIT),
					(dave(), 40_000 * EXISTENTIAL_DEPOSIT),
					(eve(), 10_000 * EXISTENTIAL_DEPOSIT),
					(ferdie(), 9_999_000 * EXISTENTIAL_DEPOSIT),
				],
			},
			vesting: orml_vesting::GenesisConfig { vesting: vec![] },
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();

		ext.execute_with(|| {
			System::set_block_number(1);
		});

		ext
	}
}

mod block_importer {
	use super::AjunaBoard;
	use ajuna_common::RunnerState;
	use ajuna_solo_runtime::{pallet_ajuna_gameregistry::Game, AccountId, GameRegistry, Runner};
	use codec::Decode;
	use frame_support::assert_ok;
	use sp_core::H256;
	use std::collections::BTreeSet;

	/// Consume block at the sidechain. We simply read the game registry storage and create the
	/// `ack_game` xt signed with the signing key
	pub fn consume_block(signing_key: AccountId) {
		let shard_id = H256::default();
		if let Some(queued_games) = GameRegistry::queued() {
			// Acknowledge game in queue with L1 as xt
			assert_ok!(GameRegistry::ack_game(
				ajuna_solo_runtime::Origin::signed(signing_key.clone()),
				queued_games.clone(),
				shard_id
			));
			// Read `ack_game` xt from current block - Not sure if this is even possible here so
			// will simulate this by reading storage directly and in the same block
			// Runner will have our game ids, we would iterate over each, decode the state and
			// execute `new_game` on the board pallet
			for game_id in queued_games {
				if let Some(RunnerState::Accepted(mut state)) = Runner::runners(game_id) {
					let game =
						Game::<AccountId>::decode(&mut state).expect("game state as accepted");
					let players = <[AccountId; 2]>::try_from(game.players)
						.expect("the game should have 2 players");
					// Call `pallet_board::new_game` with players from game
					assert_ok!(AjunaBoard::new_game(
						crate::sidechain::Origin::signed(signing_key.clone()),
						game_id,
						BTreeSet::from(players),
					));
				}
			}
		}
	}
}
