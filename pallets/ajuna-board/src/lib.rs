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

use ajuna_common::{Finished, TurnBasedGame};
use codec::Codec;
use frame_support::pallet_prelude::*;
pub use pallet::*;
use sp_std::collections::btree_set::BTreeSet;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod guessing;

/// The state of the board game
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct BoardGame<BoardId, State, Players> {
	board_id: BoardId,
	/// Players in the game
	players: Players,
	/// The current state of the game
	pub state: State,
}

impl<BoardId, State, Players> BoardGame<BoardId, State, Players> {
	/// Create a BoardGame
	fn new(board_id: BoardId, players: Players, state: State) -> Self {
		Self { board_id, players, state }
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::{ensure_signed, pallet_prelude::OriginFor};
	use sp_runtime::traits::AtLeast32BitUnsigned;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Board id
		type BoardId: Copy + Default + AtLeast32BitUnsigned + Parameter + MaxEncodedLen;
		/// A Turn for the game
		type PlayersTurn: Member + Parameter;
		/// The state of the board
		type GameState: Codec + TypeInfo + MaxEncodedLen + Clone;
		/// A turn based game
		type Game: TurnBasedGame<
			Player = Self::AccountId,
			Turn = Self::PlayersTurn,
			State = Self::GameState,
		>;
		/// Maximum number of players
		#[pallet::constant]
		type MaxNumberOfPlayers: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Game has been created
		GameCreated { board_id: T::BoardId, players: Vec<T::AccountId> },
		/// Game has finished with the winner
		GameFinished { board_id: T::BoardId, winner: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not enough players for the game
		NotEnoughPlayers,
		/// Duplicate player found
		DuplicatePlayer,
		/// Too many players
		TooManyPlayers,
		/// Invalid state from game
		InvalidStateFromGame,
		/// Player not playing
		NotPlaying,
		/// Invalid turn played
		InvalidTurn,
		/// Invalid board
		InvalidBoard,
		/// Player already in game
		PlayerAlreadyInGame,
		/// Board already exists
		BoardExists,
	}

	type BoundedPlayersOf<T> =
		BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxNumberOfPlayers>;

	type BoardGameOf<T> =
		BoardGame<<T as Config>::BoardId, <T as Config>::GameState, BoundedPlayersOf<T>>;

	type PlayersOf<T> = BTreeSet<<T as frame_system::Config>::AccountId>;

	/// Board states by board id
	#[pallet::storage]
	pub type BoardStates<T: Config> = StorageMap<_, Identity, T::BoardId, BoardGameOf<T>>;

	/// The board winners by board id
	#[pallet::storage]
	pub type BoardWinners<T: Config> = StorageMap<_, Identity, T::BoardId, T::AccountId>;

	/// Players in boards
	#[pallet::storage]
	pub type PlayerBoards<T: Config> = StorageMap<_, Identity, T::AccountId, T::BoardId>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new game with a set of players.
		/// Players are unique and would not yet be in an existing game
		#[pallet::weight(10_000)]
		pub fn new_game(
			origin: OriginFor<T>,
			board_id: T::BoardId,
			players: PlayersOf<T>,
		) -> DispatchResult {
			// TODO - This could be a whitelist based on a custom origin
			// There is potentially more than one attack vector here as anyone could assign any
			// account to a board and hence block them from playing in a legitimate game
			// As this would be ran in L2 we may want to check that we are in L2??
			let _ = ensure_signed(origin)?;

			// Ensure we have players
			ensure!(!players.is_empty(), Error::<T>::NotEnoughPlayers);
			ensure!(!BoardStates::<T>::contains_key(board_id), Error::<T>::BoardExists);

			let player_len = players.len();
			let players = BoundedPlayersOf::<T>::try_from(
				players
					.iter()
					// Ensure that this player isn't already in a game
					.filter(|player| !PlayerBoards::<T>::contains_key(player))
					.cloned()
					.collect::<Vec<T::AccountId>>(),
			)
			.map_err(|_| Error::<T>::TooManyPlayers)?;

			// If we have new players this will be the same based on the filter
			ensure!(player_len == players.len(), Error::<T>::PlayerAlreadyInGame);

			let state = T::Game::init(&players).ok_or(Error::<T>::InvalidStateFromGame)?;

			players.iter().for_each(|player| {
				PlayerBoards::<T>::insert(player, board_id);
			});

			let board_game = BoardGameOf::<T>::new(board_id, players.clone(), state);

			BoardStates::<T>::insert(board_id, board_game);

			Self::deposit_event(Event::GameCreated { board_id, players: players.into_inner() });

			Ok(())
		}

		/// Play a turn in the game for signing player
		/// If the turn produces a winner the state of the game will be removed and
		/// `Event::GameFinished` would be deposited.
		#[pallet::weight(10_000)]
		pub fn play_turn(origin: OriginFor<T>, turn: T::PlayersTurn) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let board_id = PlayerBoards::<T>::get(sender.clone()).ok_or(Error::<T>::NotPlaying)?;

			BoardStates::<T>::mutate(board_id, |maybe_board_game| match maybe_board_game {
				Some(board_game) => {
					board_game.state = T::Game::play_turn(sender, board_game.state.clone(), turn)
						.ok_or(Error::<T>::InvalidTurn)?;

					if let Finished::Winner::<T::AccountId>(winner) =
						T::Game::is_finished(&board_game.state)
					{
						// Cache result in storage, this would be cleared on `flush_winner`
						BoardWinners::<T>::insert(board_id, winner.clone());

						Self::deposit_event(Event::GameFinished { board_id, winner });
					}
					Ok(())
				},
				None => Err(Error::<T>::InvalidBoard.into()),
			})
		}

		/// Finish a board game from the pallet
		/// A board remains after finishing in BoardWinners.  Those players in that board are locked
		/// until the game is finished
		#[pallet::weight(10_000)]
		pub fn finish_game(origin: OriginFor<T>, board_id: T::BoardId) -> DispatchResult {
			// TODO if this is L2 do we really need to check the origin?
			let _ = ensure_signed(origin)?;
			// Free players to play another game
			BoardStates::<T>::get(board_id)
				.ok_or(Error::<T>::InvalidBoard)?
				.players
				.iter()
				.for_each(|player| {
					PlayerBoards::<T>::remove(player);
				});
			// Unlock board
			BoardStates::<T>::remove(board_id);
			// Clear winner
			BoardWinners::<T>::remove(board_id);
			Ok(())
		}
	}
}
