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

pub struct Logic {}

impl Logic {
	pub fn full(board: [[u8; 6]; 7]) -> bool {
		for y in board {
			for x in y {
				if x == 0 {
					return false
				}
			}
		}
		true
	}

	pub fn evaluate(board: [[u8; 6]; 7], player: u8) -> bool {
		// horizontalCheck
		for y in 0..board[0].len() {
			for x in 0..board.len() - 3 {
				if board[x][y] == player &&
					board[x + 1][y] == player &&
					board[x + 2][y] == player &&
					board[x + 3][y] == player
				{
					return true
				}
			}
		}

		// verticalCheck
		for y in 0..board[0].len() - 3 {
			for board_x in board {
				if board_x[y] == player &&
					board_x[y + 1] == player &&
					board_x[y + 2] == player &&
					board_x[y + 3] == player
				{
					return true
				}
			}
		}

		// ascendingDiagonalCheck
		for y in 0..board[0].len() - 3 {
			for x in 3..board.len() {
				if board[x][y] == player &&
					board[x - 1][y + 1] == player &&
					board[x - 2][y + 2] == player &&
					board[x - 3][y + 3] == player
				{
					return true
				}
			}
		}

		// descendingDiagonalCheck
		for y in 3..board[0].len() {
			for x in 3..board.len() {
				if board[x][y] == player &&
					board[x - 1][y - 1] == player &&
					board[x - 2][y - 2] == player &&
					board[x - 3][y - 3] == player
				{
					return true
				}
			}
		}
		false
	}

	pub fn add_stone(board: &mut [[u8; 6]; 7], column: u8, player: u8) -> bool {
		if board[column as usize][0] > 0 {
			return false
		}
		let board_rows: usize = board[0].len();
		for y in 0..board_rows {
			let y_pos = board_rows - y - 1;
			if board[column as usize][y_pos] > 0 {
				continue
			}
			board[column as usize][y_pos] = player;
			break
		}
		true
	}

	// pub fn random_board() ->  [[u8; 6]; 7] {
	//     let mut board = [[0u8; 6]; 7];
	//     let mut rng = rand::thread_rng();
	//     let mut n1: u8 = rng.gen();
	//     n1 = n1 % 42;
	//     // add randomly stones to the board
	//     for i in 0..n1 {
	//         let mut stone_set = true;
	//         loop {
	//             let mut column: u8 = rng.gen();
	//             column = column % 7;
	//             if Self::add_stone(&mut board, column, (i%2) + 1) {
	//                 break;
	//             }
	//         }
	//     }
	//     board
	// }

	// pub fn print_board(board: [[u8; 6]; 7]) {
	//     println!("   c0  c1  c2  c3  c4  c5  c6  ");
	//     println!("  + - + - + - + - + - + - + - +");
	//     for y in 0..board[0].len() {
	//         print!("r{:?}|", y);
	//         for x in 0..board.len() {
	//             match board[x][y] {
	//                 0 => print!("   |"),
	//                 1 => print!(" X |"),
	//                 2 => print!(" O |"),
	//                 _ => print!(" . |"),
	//             }
	//         }
	//         println!("");
	//         println!("  + - + - + - + - + - + - + - +");
	//     }
	// }
}

// fn main() {

//     loop {
//         let board = Logic::random_board();
//         if Logic::evaluate(board, 1) || Logic::evaluate(board, 2) {
//             println!("WE HAVE A WINNER !!!");
//             Logic::print_board(board);
//             break;
//         }
//     }
// }

#[cfg(test)]
mod tests {
	use super::*;

	const PLAYER: u8 = 1;

	#[test]
	fn full_returns_true_when_board_is_full() {
		let board: [[u8; 6]; 7] = [[PLAYER; 6]; 7];
		assert!(Logic::full(board))
	}

	#[test]
	fn full_returns_false_when_board_is_empty() {
		let board: [[u8; 6]; 7] = Default::default();
		assert!(!Logic::full(board))
	}

	#[test]
	fn full_returns_false_when_board_is_partially_filled() {
		let mut board: [[u8; 6]; 7] = Default::default();
		board[1][2] = 3;
		board[4][5] = 6;
		assert!(!Logic::full(board))
	}

	#[test]
	fn evaluate_returns_true_when_checked_horizontally() {
		/*
		Horizontally connected player like below should pass:
			y[0]: [1, 0, 0, 0, 0, 0]
			y[1]: [1, 0, 0, 0, 0, 0]
			y[2]: [1, 0, 0, 0, 0, 0]
			y[3]: [1, 0, 0, 0, 0, 0]
			y[4]: [0, 0, 0, 0, 0, 0]
			y[5]: [0, 0, 0, 0, 0, 0]
			y[6]: [0, 0, 0, 0, 0, 0]
		*/
		let mut board: [[u8; 6]; 7] = Default::default();
		for y in 0..board[0].len() {
			for x in 0..board.len() - 3 {
				board[x][y] = PLAYER;
				board[x + 1][y] = PLAYER;
				board[x + 2][y] = PLAYER;
				board[x + 3][y] = PLAYER;
				assert!(Logic::evaluate(board, PLAYER));
				board = Default::default(); // reset board
			}
		}
	}

	#[test]
	fn evaluate_returns_true_when_checked_vertically() {
		/*
		Vertically connected player like below should pass:
			y[0]: [1, 1, 1, 1, 0, 0]
			y[1]: [0, 0, 0, 0, 0, 0]
			y[2]: [0, 0, 0, 0, 0, 0]
			y[3]: [0, 0, 0, 0, 0, 0]
			y[4]: [0, 0, 0, 0, 0, 0]
			y[5]: [0, 0, 0, 0, 0, 0]
			y[6]: [0, 0, 0, 0, 0, 0]
		*/
		let mut board: [[u8; 6]; 7] = Default::default();
		for y in 0..board.len() {
			board[y] = [1, 1, 1, 1, 0, 0];
			assert!(Logic::evaluate(board, PLAYER));
			board[y] = [0, 1, 1, 1, 1, 0];
			assert!(Logic::evaluate(board, PLAYER));
			board[y] = [0, 0, 1, 1, 1, 1];
			assert!(Logic::evaluate(board, PLAYER));
		}
	}

	#[test]
	fn evaluate_returns_true_when_checked_ascending_diagonally() {
		/*
		Diagonally connected (ascending) player like below should pass:
			y[0]: [0, 0, 0, 1, 0, 0]
			y[1]: [0, 0, 1, 0, 0, 0]
			y[2]: [0, 1, 0, 0, 0, 0]
			y[3]: [1, 0, 0, 0, 0, 0]
			y[4]: [0, 0, 0, 0, 0, 0]
			y[5]: [0, 0, 0, 0, 0, 0]
			y[6]: [0, 0, 0, 0, 0, 0]
		*/
		let mut board: [[u8; 6]; 7] = Default::default();
		for y in 0..board[0].len() - 3 {
			for x in 3..board.len() {
				board[x][y] = PLAYER;
				board[x - 1][y + 1] = PLAYER;
				board[x - 2][y + 2] = PLAYER;
				board[x - 3][y + 3] = PLAYER;
				assert!(Logic::evaluate(board, PLAYER));
				board = Default::default(); // reset board
			}
		}
	}

	#[test]
	fn evaluate_returns_true_when_checked_descending_diagonally() {
		/*
		Diagonally connected (descending) player like below should pass:
			y[0]: [1, 0, 0, 0, 0, 0]
			y[1]: [0, 1, 0, 0, 0, 0]
			y[2]: [0, 0, 1, 0, 0, 0]
			y[3]: [0, 0, 0, 1, 0, 0]
			y[4]: [0, 0, 0, 0, 0, 0]
			y[5]: [0, 0, 0, 0, 0, 0]
			y[6]: [0, 0, 0, 0, 0, 0]
		*/
		let mut board: [[u8; 6]; 7] = Default::default();
		for y in 3..board[0].len() {
			for x in 3..board.len() {
				board[x][y] = PLAYER;
				board[x - 1][y - 1] = PLAYER;
				board[x - 2][y - 2] = PLAYER;
				board[x - 3][y - 3] = PLAYER;
				assert!(Logic::evaluate(board, PLAYER));
				board = Default::default(); // reset board
			}
		}
	}
}
