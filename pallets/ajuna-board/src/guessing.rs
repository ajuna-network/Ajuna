// Rules
// One player can only have one go at a time
// It's a guessing game where a player has to guess the right number
// Initial state will have this number

use crate::{self as pallet_ajuna_board};
use ajuna_common::TurnBasedGame;
use codec::{Decode, Encode};
use frame_support::{pallet_prelude::MaxEncodedLen, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use std::marker::PhantomData;

pub const THE_NUMBER: Guess = 42;
pub type Guess = u32;
pub struct MockGame<Account>(PhantomData<Account>);

const MAX_PLAYERS: usize = 2;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
pub struct GameState<Account>
where
	Account: Copy + Default + Encode + Decode + sp_std::fmt::Debug,
{
	pub players: [Account; MAX_PLAYERS],
	pub next_player: u8,
	pub solution: Guess,
	pub winner: Option<Account>,
}

impl<Account> TurnBasedGame for MockGame<Account>
where
	Account: Copy + Default + Encode + Decode + sp_std::fmt::Debug + PartialEq,
{
	type Turn = Guess;
	type Player = Account;
	type State = GameState<Self::Player>;

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
