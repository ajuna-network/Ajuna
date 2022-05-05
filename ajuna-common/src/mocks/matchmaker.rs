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

#[macro_export]
macro_rules! impl_mock_matchmaker {
	($player:ty) => {
		use std::cell::RefCell;
		use $crate::{Bracket, MatchMaker};
		pub struct MockMatchMaker;

		thread_local! {
			pub static PLAYERS: RefCell<Vec<$player>> = Default::default();
		}

		impl MatchMaker for MockMatchMaker {
			type Player = $player;

			fn enqueue(account_id: Self::Player, _bracket: Bracket) -> bool {
				if !Self::is_queued(&account_id) {
					PLAYERS.with(|cell| cell.borrow_mut().push(account_id));
					return true
				}

				false
			}

			fn clear_queue(_bracket: Bracket) {
				PLAYERS.with(|cell| cell.borrow_mut().clear());
			}

			fn is_queued(account_id: &Self::Player) -> bool {
				PLAYERS.with(|cell| {
					cell.borrow().iter().find(|&player| player == account_id).is_some()
				})
			}

			fn queued_players(_bracket: Bracket) -> Vec<Self::Player> {
				PLAYERS.with(|cell| cell.borrow().clone())
			}

			fn try_match(_bracket: Bracket, number_required: u8) -> Option<Vec<Self::Player>> {
				let len = PLAYERS.with(|cell| cell.borrow().len());
				if len < number_required as usize {
					return None
				}

				PLAYERS.with(|cell| {
					Some(cell.borrow_mut().iter().take(number_required as usize).cloned().collect())
				})
			}
		}
	};
}
