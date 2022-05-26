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
use codec::Codec;
use frame_support::pallet_prelude::*;
pub use pallet::*;
use ajuna_common::{TurnBasedGame, Finished};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct BoardGame<State, Players> {
	players: Players,
	pub state: State,
}

impl<State, Players> BoardGame<State, Players> {
	fn new(players: Players, state: State) -> Self {
		Self { players, state }
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::{ensure_signed, pallet_prelude::OriginFor};
	use sp_runtime::traits::{AtLeast32BitUnsigned, One};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Board id
		type BoardId: Copy + Default + AtLeast32BitUnsigned + Parameter + MaxEncodedLen;
		type Turn: Member + Parameter;
		type State: Codec + TypeInfo + MaxEncodedLen + Clone;
		/// A turn based game
		type Game: TurnBasedGame<Player = Self::AccountId, Turn = Self::Turn, State = Self::State>;
		/// Maximum number of players
		#[pallet::constant]
		type MaxNumberOfPlayers: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Game has been created
		GameCreated { board_id: T::BoardId, players: Vec<T::AccountId> },
		/// Game has finished with the winner
		GameFinished { winner: T::AccountId },
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
	}

	type BoundedPlayersOf<T> =
		BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxNumberOfPlayers>;

	type BoardGameOf<T> = BoardGame<<T as Config>::State, BoundedPlayersOf<T>>;

	#[pallet::storage]
	pub type BoardStates<T: Config> = StorageMap<_, Identity, T::BoardId, BoardGameOf<T>>;

	#[pallet::storage]
	pub type PlayerBoards<T: Config> = StorageMap<_, Identity, T::AccountId, T::BoardId>;

	#[pallet::storage]
	pub type BoardIdCounter<T: Config> = StorageValue<_, T::BoardId, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new game with a set of players.
		#[pallet::weight(10_000)]
		pub fn new_game(origin: OriginFor<T>, players: Vec<T::AccountId>) -> DispatchResult {
			// TODO - This could be a whitelist based on a custom origin
			// There is potentially more than one attack vector here as anyone could assign any
			// account to a board and hence block them from playing in a legitimate game
			let _ = ensure_signed(origin)?;
			// Ensure we have players
			ensure!(!players.is_empty(), Error::<T>::NotEnoughPlayers);
			// Just remove duplicates and continue
			let mut players = players;
			players.dedup();
			// Ensure we don't have too many players
			let player_len = players.len();
			ensure!(
				player_len <= T::MaxNumberOfPlayers::get() as usize,
				Error::<T>::TooManyPlayers
			);
			// Ensure that this player isn't already in a game
			let players = players
				.iter()
				.filter(|player| !PlayerBoards::<T>::contains_key(player))
				.cloned()
				.collect::<Vec<T::AccountId>>();

			ensure!(player_len == players.len(), Error::<T>::PlayerAlreadyInGame);

			let state = T::Game::init(&players).ok_or(Error::<T>::InvalidStateFromGame)?;
			let next_board_id = BoardIdCounter::<T>::mutate(|counter| {
				*counter += One::one();
				*counter
			});

			let bounded_players = BoundedPlayersOf::<T>::try_from(players.clone())
				.expect("should be a list of valid players");

			bounded_players.iter().for_each(|player| {
				PlayerBoards::<T>::insert(player, next_board_id);
			});

			let board_game = BoardGameOf::<T>::new(bounded_players, state);

			BoardStates::<T>::insert(next_board_id, board_game);

			Self::deposit_event(Event::GameCreated { board_id: next_board_id, players });

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn play_turn(origin: OriginFor<T>, turn: T::Turn) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let board_id = PlayerBoards::<T>::get(sender.clone()).ok_or(Error::<T>::NotPlaying)?;

			return BoardStates::<T>::mutate(board_id, |maybe_board_game| match maybe_board_game {
				Some(board_game) => {
					board_game.state = T::Game::play_turn(sender, board_game.state.clone(), turn)
						.ok_or(Error::<T>::InvalidTurn)?;

					if let Finished::Winner::<T::AccountId>(winner) =
						T::Game::is_finished(&board_game.state)
					{
						board_game.players.iter().for_each(|player| {
							PlayerBoards::<T>::remove(player);
						});

						BoardStates::<T>::remove(board_id);

						Self::deposit_event(Event::GameFinished { winner });
					}
					Ok(())
				},
				None => Err(Error::<T>::InvalidBoard.into()),
			})
		}

		
	}
}
