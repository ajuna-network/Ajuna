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

use ajuna_common::{Finished, TurnBasedGame};
use dot4gravity::Game as Dot4Gravity;
pub use dot4gravity::{Coordinates, GameState, Side};
use frame_support::pallet_prelude::*;
use sp_std::borrow::ToOwned;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
pub enum Turn {
	DropBomb(Coordinates),
	DropStone((Side, u8)),
}

pub struct Game<Account>(PhantomData<Account>);
impl<Account> TurnBasedGame for Game<Account>
where
	Account: Parameter,
{
	type Turn = Turn;
	type Player = Account;
	type State = GameState<Account>;

	fn init(players: &[Self::Player], seed: Option<u32>) -> Option<Self::State> {
		if let [player_1, player_2] = players {
			Some(Dot4Gravity::new_game(player_1.to_owned(), player_2.to_owned(), seed))
		} else {
			None
		}
	}

	fn get_next_player(state: &Self::State) -> Self::Player {
		state.next_player.clone()
	}

	fn play_turn(
		player: Self::Player,
		state: Self::State,
		turn: Self::Turn,
	) -> Option<Self::State> {
		match turn {
			Turn::DropBomb(coords) => Dot4Gravity::drop_bomb(state, coords, player),
			Turn::DropStone((side, pos)) => Dot4Gravity::drop_stone(state, player, side, pos),
		}
		.ok()
	}

	fn abort(state: Self::State, winner: Self::Player) -> Self::State {
		let mut state = state;
		state.winner = Some(winner);
		state
	}

	fn is_finished(state: &Self::State) -> Finished<Self::Player> {
		match state.winner.clone() {
			Some(winner) => Finished::Winner(winner),
			None => Finished::No,
		}
	}

	fn seed(state: &Self::State) -> Option<u32> {
		Some(state.seed)
	}
}

// allow to reduce unnecessary vertical space
#[allow(unused_must_use)]
#[cfg(test)]
mod tests {
	use crate::{
		self as pallet_ajuna_board, dot4gravity::*, mock::new_test_ext, BTreeSet, BoardStates,
		BoardWinners, Error, Seed,
	};
	use frame_support::{assert_noop, assert_ok};
	use frame_system::mocking::{MockBlock, MockUncheckedExtrinsic};
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};

	const ALICE: u32 = 1;
	const BOB: u32 = 2;
	const CHARLIE: u32 = 3;
	const BOARD_ID: u32 = 1;

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
		type AccountId = u32;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = frame_support::traits::ConstU64<250>;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = frame_support::traits::ConstU16<42>;
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	impl pallet_ajuna_board::Config for Test {
		type Event = Event;
		type BoardId = u32;
		type PlayersTurn = crate::dot4gravity::Turn;
		type GameState = crate::dot4gravity::GameState<MockAccountId>;
		type Game = crate::dot4gravity::Game<MockAccountId>;
		type MaxNumberOfPlayers = frame_support::traits::ConstU32<2>;
		type MaxNumberOfIdleBlocks = frame_support::traits::ConstU32<10>;
		type MaxNumberOfGamesToExpire = frame_support::traits::ConstU32<5>;
		type WeightInfo = ();
	}

	#[test]
	fn should_create_new_game() {
		new_test_ext().execute_with(|| {
			assert_ok!(AjunaBoard::new_game(
				Origin::signed(ALICE),
				BOARD_ID,
				BTreeSet::from([BOB, CHARLIE])
			));
			System::assert_last_event(Event::AjunaBoard(crate::Event::GameCreated {
				board_id: BOARD_ID,
				players: vec![BOB, CHARLIE],
			}));
			assert!(BoardStates::<Test>::contains_key(BOARD_ID));
			assert!(Seed::<Test>::get().is_none());

			let tests_for_errors = vec![
				(BTreeSet::from([BOB, CHARLIE]), BOARD_ID, Error::<Test>::BoardExists),
				(BTreeSet::from([ALICE, BOB]), 11, Error::<Test>::PlayerAlreadyInGame),
				(BTreeSet::from([ALICE, CHARLIE]), 22, Error::<Test>::PlayerAlreadyInGame),
				// TODO: should we early reject with NotEnoughPlayer?
				(BTreeSet::from([ALICE]), 33, Error::<Test>::InvalidStateFromGame),
				// TODO: should we early reject with NotEnoughPlayer?
				(BTreeSet::from([BOB]), 44, Error::<Test>::PlayerAlreadyInGame),
				(BTreeSet::from([CHARLIE]), 44, Error::<Test>::PlayerAlreadyInGame),
				// TODO: should we early reject with TooManyPlayers?
				(BTreeSet::from([ALICE, BOB, CHARLIE]), 55, Error::<Test>::PlayerAlreadyInGame),
			];
			for (players, board_id, expected_error) in tests_for_errors {
				assert_noop!(
					AjunaBoard::new_game(Origin::signed(ALICE), board_id, players),
					expected_error
				);
			}
		})
	}

	#[test]
	fn should_play_and_mutate_game_state() {
		new_test_ext().execute_with(|| {
			let test_coord = Coordinates::new(0, 0);
			AjunaBoard::new_game(Origin::signed(ALICE), BOARD_ID, BTreeSet::from([BOB, CHARLIE]));
			let board_state_before = BoardStates::<Test>::get(BOARD_ID).unwrap();

			assert_ok!(AjunaBoard::play_turn(Origin::signed(CHARLIE), Turn::DropBomb(test_coord)));
			let board_state_after = BoardStates::<Test>::get(BOARD_ID).unwrap();

			assert_ne!(board_state_before.state.board, board_state_after.state.board);
			assert_eq!(board_state_before.state.bombs, [(BOB, 3), (CHARLIE, 3)]);
			assert_eq!(board_state_after.state.bombs, [(BOB, 3), (CHARLIE, 3 - 1)]);
		});
	}

	#[test]
	fn should_play_turn_and_finish_game() {
		new_test_ext().execute_with(|| {
			AjunaBoard::new_game(Origin::signed(ALICE), BOARD_ID, BTreeSet::from([BOB, CHARLIE]));
			assert_noop!(
				AjunaBoard::play_turn(
					Origin::signed(ALICE),
					Turn::DropBomb(Coordinates::new(0, 0))
				),
				Error::<Test>::NotPlaying
			);
			assert!(Seed::<Test>::get().is_none());

			// drop from left to right to trigger the winning condition due to randomized board
			for i in 0..10 {
				let bomb_coord = Coordinates::new(i as u8, i as u8);
				// Bomb phase
				AjunaBoard::play_turn(Origin::signed(BOB), Turn::DropBomb(bomb_coord));
				AjunaBoard::play_turn(Origin::signed(CHARLIE), Turn::DropBomb(bomb_coord));
				// Stone phase
				AjunaBoard::play_turn(Origin::signed(BOB), Turn::DropStone((Side::North, 0)));
				AjunaBoard::play_turn(Origin::signed(CHARLIE), Turn::DropStone((Side::North, 1)));

				if let Some(Event::AjunaBoard(crate::Event::GameFinished { board_id, winner })) =
					System::events().into_iter().map(|r| r.event).last()
				{
					assert_eq!(board_id, BOARD_ID);
					assert_ne!(winner, ALICE);
					assert!(winner == BOB || winner == CHARLIE);
					assert_eq!(BoardWinners::<Test>::get(BOARD_ID), Some(winner));
					assert!(Seed::<Test>::get().is_some());
					break
				}
			}

			System::assert_last_event(Event::AjunaBoard(crate::Event::GameFinished {
				board_id: BOARD_ID,
				winner: CHARLIE,
			}));
			assert_ok!(AjunaBoard::finish_game(Origin::signed(ALICE), BOARD_ID));
			assert!(BoardStates::<Test>::get(BOARD_ID).is_none());
			assert!(BoardWinners::<Test>::get(BOARD_ID).is_none());
		})
	}
}
