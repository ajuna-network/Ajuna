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

use ajuna_common::{Bracket, MatchMaker};
use frame_support::pallet_prelude::*;
pub use pallet::*;
use sp_std::{marker::PhantomData, prelude::*};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use ajuna_common::{Bracket, BracketCounter};

	///A range describing the start and end points of the bracket, a rudimentary FIFO
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct BracketRange {
		pub start: BracketCounter,
		pub end: BracketCounter,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Map of brackets with their index
	#[pallet::storage]
	pub type Brackets<T: Config> =
		StorageMap<_, Blake2_128Concat, Bracket, BracketRange, ValueQuery>;

	/// A double map indexed by bracket and account
	#[pallet::storage]
	pub type Players<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Bracket,
		Blake2_128Concat,
		BracketCounter,
		T::AccountId,
		OptionQuery,
	>;

	/// A map tracking which accounts are queued
	#[pallet::storage]
	pub type PlayerQueue<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u8, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Player is queued for a match.
		Queued(T::AccountId),
		/// Players are now matched and have been removed from the queue
		Matched(Vec<T::AccountId>),
	}
}

pub struct MatchMaking<T>(PhantomData<T>);

impl<T: Config> MatchMaker for MatchMaking<T> {
	type Player = T::AccountId;

	fn enqueue(account_id: Self::Player, bracket: Bracket) -> bool {
		if Self::is_queued(&account_id) {
			return false
		}

		Brackets::<T>::mutate(bracket, |range| {
			Players::<T>::insert(bracket, range.end, account_id.clone());
			range.end += 1;
			PlayerQueue::<T>::insert(account_id.clone(), 1);

			Pallet::<T>::deposit_event(Event::Queued(account_id));
			true
		})
	}

	fn clear_queue(bracket: Bracket) {
		Players::<T>::iter_prefix_values(bracket).for_each(|account_id| {
			PlayerQueue::<T>::remove(account_id);
		});
		Players::<T>::remove_prefix(bracket, None);
	}

	fn is_queued(account_id: &Self::Player) -> bool {
		PlayerQueue::<T>::contains_key(account_id.clone())
	}

	fn queued_players(bracket: Bracket) -> Vec<Self::Player> {
		Players::<T>::iter_prefix_values(bracket).collect()
	}

	fn try_match(bracket: Bracket, number_required: u8) -> Option<Vec<Self::Player>> {
		if Players::<T>::iter_prefix_values(bracket).count() < number_required as usize {
			return None
		}

		let players = (0..number_required)
			.into_iter()
			.filter_map(|index| {
				let index = index as u32 + Brackets::<T>::get(bracket).start;
				Players::<T>::take(bracket, index)
			})
			.map(|player| {
				PlayerQueue::<T>::remove(player.clone());
				player
			})
			.collect::<Vec<_>>();

		Brackets::<T>::mutate(bracket, |range| {
			range.start += players.len() as u32;
		});

		Pallet::<T>::deposit_event(Event::Matched(players.clone()));

		Some(players)
	}
}
