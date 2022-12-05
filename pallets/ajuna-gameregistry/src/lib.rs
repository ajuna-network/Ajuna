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

pub use pallet::*;

use codec::{Decode, Encode, Input};
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::{Member, *},
	sp_runtime::traits::{AtLeast32BitUnsigned, Dispatchable},
	traits::{schedule::Named, Contains},
	Parameter, RuntimeDebug,
};
use frame_system::pallet_prelude::*;
use pallet_ajuna_matchmaker::{MatchMaker, DEFAULT_BRACKET, DEFAULT_PLAYERS};
use sp_std::vec::Vec;

mod types;
use types::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Proposal: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ From<Call<Self>>;

		type Scheduler: Named<Self::BlockNumber, Self::Proposal, Self::PalletsOrigin>;

		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

		/// Matchmaker will handle game queueing and matchups
		type MatchMaker: MatchMaker<Player = Self::AccountId>;

		/// An identifier for a game, we use the runner identifier
		type GameId: Identifier;

		/// Generate identifiers for games
		type GetIdentifier: GetIdentifier<Self::GameId>;

		/// The Runners
		type Runner: Runner<RunnerId = Self::GameId>;

		/// Authenticated TEE's
		type Observers: Contains<Self::AccountId>;

		/// Identifier for a Shard
		type ShardIdentifier: Member + Parameter;

		#[pallet::constant]
		/// The maximum number of games that can be acknowledged in one batch
		type MaxAcknowledgeBatch: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::event]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		/// Too many games trying to be acknowledged in batch.
		AcknowledgeBatchTooLarge,
		/// There is no such game entry.
		NoGameEntry,
		/// Player is already queued for a match.
		AlreadyQueued,
		/// Invalid winner
		InvalidWinner,
		/// Not Signed by an Observer
		NotSignedByObserver,
		/// Invalid payload
		InvalidPayload,
		/// Invalid game state
		InvalidGameState,
		/// Failed to queue
		FailedToQueue,
		/// Already playing
		AlreadyPlaying,
	}

	#[pallet::storage]
	#[pallet::getter(fn queued)]
	pub type Queued<T: Config> = StorageValue<_, Vec<T::GameId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub type Players<T: Config> = StorageMap<_, Blake2_128, T::AccountId, T::GameId, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Queue sender for a game
		/// We also use this as an opportunity to match a player and set off a runner for the game
		#[pallet::weight(12_345)]
		pub fn queue(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Players::<T>::contains_key(who.clone()), Error::<T>::AlreadyPlaying);
			// Queue sender as player
			ensure!(T::MatchMaker::enqueue(who, DEFAULT_BRACKET), Error::<T>::AlreadyQueued);
			// Let's process a match, *may* not include this player based on the queue but we
			// get this work paid for by the players and not the community
			if let Some(players) = T::MatchMaker::try_match(DEFAULT_BRACKET, DEFAULT_PLAYERS) {
				// Create the game to be run, we have players and will wait on the game being
				// accepted and with that create runner with configuration, basically the players at
				// the moment
				let identifier = T::Runner::create::<T::GetIdentifier>(
					Game::new(players.clone()).encode().into(),
				)
				.ok_or(Error::<T>::FailedToQueue)?;

				// Players need to know which game they are in
				players.iter().for_each(|player| {
					Players::<T>::insert(player, identifier);
				});

				// Locally store the queued runner.
				// This is used by L2 to prepare for `ack_game`. We will probably want to review
				// this when we have multiple shards but until then this will suffice
				Queued::<T>::append(identifier);
			}
			Ok(())
		}

		/// Drop game will remove the game from the registry
		#[pallet::weight(12_345)]
		pub fn drop_game(origin: OriginFor<T>, game_id: T::GameId) -> DispatchResult {
			let _who: T::AccountId = frame_system::ensure_signed(origin)?;
			// Ensure this is signed by an observer
			// TODO: reinstate this after we have a way of adding observers via Teerex
			// ensure!(T::Observers::contains(&who), Error::<T>::NotSignedByObserver);

			// We silently remove the game id whether it exists or not
			T::Runner::remove(&game_id)?;

			Ok(())
		}

		/// Acknowledge a set of games
		#[pallet::weight(12_345)]
		pub fn ack_game(
			origin: OriginFor<T>,
			game_ids: Vec<T::GameId>,
			_shard_id: T::ShardIdentifier,
		) -> DispatchResult {
			let who: T::AccountId = frame_system::ensure_signed(origin)?;
			// Ensure this is signed by an observer
			// TODO: reinstate this after we have a way of adding observers via Teerex
			// ensure!(T::Observers::contains(&who), Error::<T>::NotSignedByObserver);

			// Ensure we aren't receiving a batch which is too big
			ensure!(
				game_ids.len() <= T::MaxAcknowledgeBatch::get() as usize,
				Error::<T>::AcknowledgeBatchTooLarge
			);

			// Run through batch and accept those that are in valid state `Queued`
			// Those that fail, fail silently
			game_ids.iter().for_each(|game_id| {
				if let Some(RunnerState::Queued(mut state)) = T::Runner::get_state(game_id) {
					if let Ok(mut game) = Game::decode(&mut state) {
						// Remove the queued item
						if let Some(mut queued) = Queued::<T>::get() {
							queued.retain(|x| x != game_id);
							Queued::<T>::put(queued);
						}
						game.tee_id = Some(who.clone());
						// Accept this game, log if we failed to accept this game
						let _ =
							T::Runner::accept(game_id, Some(game.encode().into())).map_err(|e| {
								log::debug!("Accepting {:?} failed with error:{:?}", game_id, e);
							});
					} else {
						log::error!("Failed to decoded state of game {:?} from state, this is unrecoverable for this game now", game_id);
					}
				} else {
					log::warn!("Game {:?} is not queued, this shouldn't be possible!", game_id);
				}
			});

			Ok(())
		}

		/// Finish game
		#[pallet::weight(12_345)]
		pub fn finish_game(
			origin: OriginFor<T>,
			game_id: T::GameId,
			winner: T::AccountId,
			_shard_id: T::ShardIdentifier,
		) -> DispatchResult {
			let _who: T::AccountId = frame_system::ensure_signed(origin)?;
			// Ensure this is signed by an observer
			// TODO: reinstate this after we have a way of adding observers via Teerex
			// ensure!(T::Observers::contains(&who), Error::<T>::NotSignedByObserver);

			// If the game is in the accepted state we can ascertain if their is a valid winner
			// and mark the game state as finished
			if let RunnerState::Accepted(mut state) =
				T::Runner::get_state(&game_id).ok_or(Error::<T>::NoGameEntry)?
			{
				let mut game = Game::decode(&mut state).map_err(|_| Error::<T>::InvalidPayload)?;

				ensure!(game.players.contains(&winner), Error::<T>::InvalidWinner);

				game.winner = Some(winner);

				// Players need to know which game they are in
				game.players.iter().for_each(|player| {
					Players::<T>::remove(player);
				});

				log::debug!("Finished game {:?}", game_id);
				T::Runner::finished(&game_id, Some(game.encode().into()))?;

				Ok(())
			} else {
				log::error!("Invalid game state for game {:?}", game_id);
				Err(Error::<T>::InvalidGameState.into())
			}
		}
	}
}
