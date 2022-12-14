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

use super::*;
use sp_std::vec::Vec;

pub const DEFAULT_BRACKET: Bracket = 0;
pub const DEFAULT_PLAYERS: u8 = 2;

/// Type to identify a bracket
pub type Bracket = u32;

/// Type to counter items in a bracket
pub type BracketCounter = u32;

/// A range describing the start and end points of the bracket, a rudimentary FIFO
#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct BracketRange {
	pub start: BracketCounter,
	pub end: BracketCounter,
}

/// A matchmaker trait which groups players as accounts in brackets
pub trait Matchmaker {
	/// The identifier for player
	type Player;

	/// Enqueue account in bracket
	fn enqueue(account_id: Self::Player, bracket: Bracket) -> bool;

	/// Clear queue for bracket
	fn clear_queue(bracket: Bracket);

	/// Is account queued already?
	fn is_queued(account_id: &Self::Player) -> bool;

	/// Those that are queued in bracket
	fn queued_players(bracket: Bracket) -> Vec<Self::Player>;

	/// Try to get a match using bracket specifying groups size.  A number of players up to
	/// `number_required` will be returned based on availability in the bracket. The players would
	/// be removed from the bracket and queue
	fn try_match(bracket: Bracket, number_required: u32) -> Option<Vec<Self::Player>>;
}

pub struct Matchmaking<T>(PhantomData<T>);
impl<T: Config> Matchmaker for Matchmaking<T> {
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
		let limit = Players::<T>::iter_prefix_values(bracket)
			.map(|account_id| {
				PlayerQueue::<T>::remove(account_id);
			})
			.count() as u32;
		let r = Players::<T>::clear_prefix(bracket, limit, None);
		if r.maybe_cursor.is_some() {
			Self::clear_queue(bracket)
		}
	}

	fn is_queued(account_id: &Self::Player) -> bool {
		PlayerQueue::<T>::contains_key(account_id.clone())
	}

	fn queued_players(bracket: Bracket) -> Vec<Self::Player> {
		Players::<T>::iter_prefix_values(bracket).collect()
	}

	fn try_match(bracket: Bracket, number_required: u32) -> Option<Vec<Self::Player>> {
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
