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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_system::{ensure_signed, pallet_prelude::*};
pub use pallet::*;
use pallet_ajuna_matchmaker::{Matchmaker, DEFAULT_BRACKET};
use sp_runtime::traits::{AtLeast32BitUnsigned, BlockNumberProvider, Saturating};
use sp_std::vec::Vec;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod dot4gravity;
mod types;
use types::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Matchmaker: Matchmaker<Player = Self::AccountId>;
		/// Board id
		type BoardId: Copy + Default + AtLeast32BitUnsigned + Parameter + MaxEncodedLen;
		/// A Turn for the game
		type PlayersTurn: Member + Parameter + From<dot4gravity::Turn>;
		/// The state of the board
		type GameState: Codec + TypeInfo + MaxEncodedLen + Clone;
		/// A turn based game
		type Game: TurnBasedGame<
			Player = Self::AccountId,
			Turn = Self::PlayersTurn,
			State = Self::GameState,
		>;
		/// Number of players required for a game.
		#[pallet::constant]
		type Players: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Game has been created
		GameCreated {
			board_id: T::BoardId,
			players: Vec<T::AccountId>,
		},
		/// Game has finished with the winner
		GameFinished {
			board_id: T::BoardId,
			winner: T::AccountId,
		},

		NoMatchFound,
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidGameState,
		InvalidTurn,
		InvalidPlayers,
		NotPlaying,
		AlreadyInGame,
		AlreadyQueued,
		UnknownBoard,
		BoardInUse,
	}

	#[pallet::storage]
	pub type NextBoardId<T: Config> = StorageValue<_, T::BoardId, ValueQuery>;

	#[pallet::storage]
	pub type BoardGames<T: Config> = StorageMap<_, Identity, T::BoardId, BoardGameOf<T>>;

	/// Players in boards
	#[pallet::storage]
	pub type PlayerBoards<T: Config> = StorageMap<_, Identity, T::AccountId, T::BoardId>;

	/// Random seed
	#[pallet::storage]
	pub type Seed<T> = StorageValue<_, u32>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(12_345)]
		pub fn queue(origin: OriginFor<T>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			ensure!(T::Matchmaker::enqueue(player, DEFAULT_BRACKET), Error::<T>::AlreadyQueued);
			if let Some(players) = T::Matchmaker::try_match(DEFAULT_BRACKET, T::Players::get()) {
				Self::create_game(players)?;
			};
			Ok(())
		}

		#[pallet::weight(12_345)]
		pub fn play(origin: OriginFor<T>, turn: T::PlayersTurn) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let board_id = PlayerBoards::<T>::get(&player).ok_or(Error::<T>::NotPlaying)?;

			let mut board_game = BoardGames::<T>::get(board_id).ok_or(Error::<T>::UnknownBoard)?;
			let new_state = T::Game::play_turn(player, board_game.state, turn)
				.ok_or(Error::<T>::InvalidTurn)?;

			if let Finished::Winner::<T::AccountId>(winner) = T::Game::is_finished(&new_state) {
				Self::finish_game(board_id, winner)?;
			} else {
				board_game.state = new_state;
				BoardGames::<T>::insert(board_id, board_game);
			}
			Ok(())
		}

		#[pallet::weight(12_345)]
		pub fn clear_board(origin: OriginFor<T>, board_id: T::BoardId) -> DispatchResult {
			ensure_root(origin)?;

			BoardGames::<T>::try_mutate_exists(board_id, |maybe_board_game| {
				if let Some(board_game) = maybe_board_game {
					ensure!(
						board_game
							.players
							.iter()
							.all(|player| PlayerBoards::<T>::get(player) != Some(board_id)),
						Error::<T>::BoardInUse
					);

					*maybe_board_game = None;

					Ok(())
				} else {
					Err(Error::<T>::UnknownBoard)
				}
			})
			.map_err(|err| err.into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn create_game(players: Vec<PlayerOf<T>>) -> DispatchResult {
		for player in &players {
			ensure!(PlayerBoards::<T>::get(player).is_none(), Error::<T>::AlreadyInGame);
		}

		let board_id = NextBoardId::<T>::get();
		let seed = Seed::<T>::get();
		let state = T::Game::init(&players, seed).ok_or(Error::<T>::InvalidGameState)?;
		Self::seed_for_next(&state);

		let bounded_players = players.clone().try_into().map_err(|_| Error::<T>::InvalidPlayers)?;
		let now = frame_system::Pallet::<T>::current_block_number();
		let board_game = BoardGameOf::<T>::new(board_id, bounded_players, state, now);

		players.iter().for_each(|player| PlayerBoards::<T>::insert(player, board_id));
		BoardGames::<T>::insert(board_id, board_game);
		NextBoardId::<T>::mutate(|board_id| board_id.saturating_inc());
		Self::deposit_event(Event::GameCreated { board_id, players });
		Ok(())
	}

	fn seed_for_next(game_state: &T::GameState) {
		match T::Game::seed(game_state) {
			Some(seed) => Seed::<T>::put(seed),
			None => Seed::<T>::kill(),
		}
	}

	fn finish_game(board_id: T::BoardId, winner: PlayerOf<T>) -> DispatchResult {
		BoardGames::<T>::get(board_id)
			.ok_or(Error::<T>::UnknownBoard)?
			.players
			.iter()
			.for_each(PlayerBoards::<T>::remove);
		Self::deposit_event(Event::GameFinished { board_id, winner });
		Ok(())
	}
}
