// Rules
// One player can only have one go at a time
// It's a guessing game where a player has to guess the right number
// Initial state will have this number

use crate::{self as pallet_ajuna_board};
use ajuna_common::TurnBasedGame;
use codec::{Decode, Encode};
use frame_support::{pallet_prelude::MaxEncodedLen, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

pub const THE_NUMBER: Guess = 42;
pub type Guess = u32;
pub struct MockGame<Account>(PhantomData<Account>);

const MAX_PLAYERS: usize = 2;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
pub struct GameState<Account>
where
	Account: Clone + Encode + Decode + sp_std::fmt::Debug,
{
	pub players: [Account; MAX_PLAYERS],
	pub next_player: u8,
	pub solution: Guess,
	pub winner: Option<Account>,
}

impl<Account> TurnBasedGame for MockGame<Account>
where
	Account: Clone + Encode + Decode + sp_std::fmt::Debug + PartialEq,
{
	type Turn = Guess;
	type Player = Account;
	type State = GameState<Self::Player>;

	fn init(players: &[Self::Player], _seed: Option<u32>) -> Option<Self::State> {
		match players.to_vec().try_into() {
			Ok(players) =>
				Some(GameState { players, next_player: 0, solution: THE_NUMBER, winner: None }),
			_ => None,
		}
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

		if state.solution == turn {
			state.winner = Some(player);
		}

		Some(state)
	}

	fn abort(state: Self::State, winner: Self::Player) -> Self::State {
		let mut state = state;
		state.winner = Some(winner);
		state
	}

	fn is_finished(state: &Self::State) -> pallet_ajuna_board::Finished<Self::Player> {
		let winner = &state.winner;
		match winner.clone() {
			None => pallet_ajuna_board::Finished::No,
			Some(winner) => pallet_ajuna_board::Finished::Winner(winner),
		}
	}

	fn seed(_state: &Self::State) -> Option<u32> {
		None
	}
}
