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
pub use types::*;

use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
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
