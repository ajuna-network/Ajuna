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

use crate::{mock::*, types::*, MatchMaking};
use frame_system::{EventRecord, Phase};

fn queue_players_sorted(bracket: Bracket) -> Vec<u32> {
	let mut players = MatchMaking::<Test>::queued_players(bracket);
	players.sort_unstable();
	players
}

#[test]
fn enqueue_should_add_player_to_specified_bracket() {
	new_test_ext().execute_with(|| {
		assert!(queue_players_sorted(BRACKET_0).is_empty());
		[PLAYER_1, PLAYER_2, PLAYER_3].into_iter().for_each(|player| {
			assert!(MatchMaking::<Test>::enqueue(player, BRACKET_0));
		});

		assert_eq!(queue_players_sorted(BRACKET_0), [PLAYER_1, PLAYER_2, PLAYER_3]);
		assert!(queue_players_sorted(BRACKET_1).is_empty());
		assert!(queue_players_sorted(BRACKET_2).is_empty());
	})
}

#[test]
fn enqueue_should_emit_events_correctly() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1); // events are emitted from block 1

		assert!(MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0));
		assert_eq!(System::events().len(), 1);
		assert_eq!(
			System::events(),
			vec![EventRecord {
				phase: Phase::Initialization,
				event: RuntimeEvent::Matchmaker(crate::Event::Queued(PLAYER_1)),
				topics: vec![],
			}]
		);

		// no new events are emitted when player cannot be queued
		assert!(!MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0));
		assert_eq!(System::events().len(), 1);
	})
}

#[test]
fn clear_queue_should_clear_specified_bracket() {
	new_test_ext().execute_with(|| {
		let players = [PLAYER_1, PLAYER_2, PLAYER_3];
		players.iter().for_each(|player| {
			assert!(MatchMaking::<Test>::enqueue(*player, BRACKET_0));
		});
		assert_eq!(queue_players_sorted(BRACKET_0), players);
		MatchMaking::<Test>::clear_queue(BRACKET_0);
		assert!(queue_players_sorted(BRACKET_0).is_empty());
	})
}

#[test]
fn clear_queue_should_not_clear_other_brackets() {
	new_test_ext().execute_with(|| {
		assert!(MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_2, BRACKET_0));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_3, BRACKET_1));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_4, BRACKET_2));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_5, BRACKET_2));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_6, BRACKET_2));

		let initial_bracket_0_size = queue_players_sorted(BRACKET_0);
		let initial_bracket_1_size = queue_players_sorted(BRACKET_1);
		let initial_bracket_2_size = queue_players_sorted(BRACKET_2);

		MatchMaking::<Test>::clear_queue(BRACKET_0);
		assert!(queue_players_sorted(BRACKET_0).is_empty());
		assert_ne!(queue_players_sorted(BRACKET_0), initial_bracket_0_size);
		assert_eq!(queue_players_sorted(BRACKET_1), initial_bracket_1_size);
		assert_eq!(queue_players_sorted(BRACKET_2), initial_bracket_2_size);
	})
}

#[test]
fn try_match_should_emit_events_correctly() {
	new_test_ext().execute_with(|| {
		[PLAYER_1, PLAYER_2, PLAYER_3, PLAYER_4].into_iter().for_each(|player| {
			assert!(MatchMaking::<Test>::enqueue(player, BRACKET_0));
		});
		System::set_block_number(1); // events are emitted from block 1

		// first match
		assert_eq!(MatchMaking::<Test>::try_match(BRACKET_0, 2), Some(vec![PLAYER_1, PLAYER_2]));
		assert_eq!(System::events().len(), 1);
		assert_eq!(
			System::events(),
			vec![EventRecord {
				phase: Phase::Initialization,
				event: RuntimeEvent::Matchmaker(crate::Event::Matched(vec![PLAYER_1, PLAYER_2])),
				topics: vec![],
			}]
		);

		// second match
		assert_eq!(MatchMaking::<Test>::try_match(BRACKET_0, 2), Some(vec![PLAYER_3, PLAYER_4]));
		assert_eq!(System::events().len(), 2);
		assert_eq!(
			System::events()[1..],
			vec![EventRecord {
				phase: Phase::Initialization,
				event: RuntimeEvent::Matchmaker(crate::Event::Matched(vec![PLAYER_3, PLAYER_4])),
				topics: vec![],
			}]
		);

		// no further events are emitted when there is no match
		assert!(MatchMaking::<Test>::try_match(BRACKET_0, 2).is_none());
		assert_eq!(System::events().len(), 2);
	})
}

#[test]
fn is_queued_should_return_true_when_player_is_already_queued() {
	new_test_ext().execute_with(|| {
		assert!(MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0));
		assert!(MatchMaking::<Test>::is_queued(&PLAYER_1));
	});
}

#[test]
fn is_queued_should_return_false_when_player_is_not_queued() {
	new_test_ext().execute_with(|| {
		assert!(!MatchMaking::<Test>::is_queued(&PLAYER_1));
		assert!(!MatchMaking::<Test>::is_queued(&PLAYER_2));
	});
}

#[test]
fn is_queued_should_return_false_when_player_leaves_queue_via_matchmaking() {
	new_test_ext().execute_with(|| {
		assert!(MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_2, BRACKET_0));
		MatchMaking::<Test>::try_match(BRACKET_0, 2);
		assert!(!MatchMaking::<Test>::is_queued(&PLAYER_1));
		assert!(!MatchMaking::<Test>::is_queued(&PLAYER_2));
	})
}

#[test]
fn queued_players_should_be_zero_when_there_are_no_queued_players() {
	new_test_ext().execute_with(|| {
		assert!(queue_players_sorted(BRACKET_0).is_empty());
		assert!(queue_players_sorted(BRACKET_1).is_empty());
		assert!(queue_players_sorted(BRACKET_2).is_empty());
	})
}

#[test]
fn test_try_duplicate_queue() {
	new_test_ext().execute_with(|| {
		assert!(queue_players_sorted(BRACKET_0).is_empty());
		assert!(MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0));
		assert!(!MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0)); // try same bracket
		assert!(!MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_1)); // try other bracket

		assert!(MatchMaking::<Test>::enqueue(PLAYER_2, BRACKET_1));
		assert!(!MatchMaking::<Test>::enqueue(PLAYER_2, BRACKET_1)); // try same bracket
		assert!(!MatchMaking::<Test>::enqueue(PLAYER_2, BRACKET_0)); // try other bracket
	});
}

#[test]
fn test_enqueue() {
	new_test_ext().execute_with(|| {
		assert!(queue_players_sorted(BRACKET_0).is_empty());
		assert!(MatchMaking::<Test>::try_match(BRACKET_0, 1).is_none());

		assert!(MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0));
		assert_eq!(queue_players_sorted(BRACKET_0), [PLAYER_1]);
		assert!(MatchMaking::<Test>::try_match(BRACKET_0, 2).is_none());

		assert!(MatchMaking::<Test>::enqueue(PLAYER_2, BRACKET_0));
		assert_eq!(queue_players_sorted(BRACKET_0), [PLAYER_1, PLAYER_2]);

		assert_eq!(MatchMaking::<Test>::try_match(BRACKET_0, 2), Some(vec![PLAYER_1, PLAYER_2]));
		assert!(queue_players_sorted(BRACKET_0).is_empty());
		assert!(MatchMaking::<Test>::try_match(BRACKET_0, 1).is_none());

		assert!(MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_2, BRACKET_0));
		assert_eq!(queue_players_sorted(BRACKET_0), [PLAYER_1, PLAYER_2]);
		MatchMaking::<Test>::clear_queue(BRACKET_0);
		assert!(MatchMaking::<Test>::try_match(BRACKET_0, 1).is_none());
		assert!(queue_players_sorted(BRACKET_0).is_empty());
	});
}

#[test]
fn test_brackets() {
	new_test_ext().execute_with(|| {
		assert!(queue_players_sorted(BRACKET_0).is_empty());

		assert!(MatchMaking::<Test>::enqueue(PLAYER_1, BRACKET_0));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_2, BRACKET_0));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_3, BRACKET_0));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_4, BRACKET_1));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_5, BRACKET_1));
		assert!(MatchMaking::<Test>::enqueue(PLAYER_6, BRACKET_2));
		assert_eq!(queue_players_sorted(BRACKET_0), [PLAYER_1, PLAYER_2, PLAYER_3]);
		assert_eq!(queue_players_sorted(BRACKET_1), [PLAYER_4, PLAYER_5]);
		assert_eq!(queue_players_sorted(BRACKET_2), [PLAYER_6]);
	});
}

#[test]
fn matchmaking_should_be_fifo() {
	new_test_ext().execute_with(|| {
		let players_per_game = 3; // 3 player game
		let total_games = 10;
		let total_players = players_per_game * total_games;

		for player in 0..total_players {
			MatchMaking::<Test>::enqueue(player, BRACKET_0);
		}

		for i in (0..total_players).step_by(players_per_game as usize) {
			let matched = MatchMaking::<Test>::try_match(BRACKET_0, players_per_game as u8);
			let expected_match = (i..(i + players_per_game)).collect();
			assert_eq!(matched, Some(expected_match));
		}
	});
}
