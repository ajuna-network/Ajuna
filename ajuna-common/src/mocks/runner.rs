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
macro_rules! impl_mock_runner {
	($id: ty) => {
		use sp_runtime::{DispatchError, DispatchResult};
		use $crate::{GetIdentifier, Runner, RunnerState, State};
		pub struct MockRunner;
		pub const GLOBAL_IDENTIFIER: u32 = 1;
		const UNKNOWN_IDENTIFIER: &'static str = "Unknown identifier";

		thread_local! {
			pub static STATE: RefCell<Option<RunnerState>> = RefCell::new(None);
		}

		// #[derive(
		// 	Encode,
		// 	Decode,
		// 	Default,
		// 	Clone,
		// 	Eq,
		// 	PartialEq,
		// 	RuntimeDebug,
		// 	frame_support::pallet_prelude::TypeInfo,
		// 	frame_support::pallet_prelude::MaxEncodedLen,
		// )]
		pub struct MockGetIdentifier;
		impl GetIdentifier<u32> for MockGetIdentifier {
			fn get_identifier() -> u32 {
				GLOBAL_IDENTIFIER
			}
		}

		impl Runner for MockRunner {
			type Identifier = u32;

			fn create<G: GetIdentifier<Self::Identifier>>(
				initial_state: State,
			) -> Option<Self::Identifier> {
				STATE.with(|cell| *cell.borrow_mut() = Some(RunnerState::Queued(initial_state)));
				G::get_identifier().into()
			}

			fn accept(identifier: Self::Identifier, new_state: Option<State>) -> DispatchResult {
				if identifier == GLOBAL_IDENTIFIER {
					STATE.with(|cell| {
						*cell.borrow_mut() =
							Some(RunnerState::Accepted(new_state.expect("some state")))
					});
					Ok(())
				} else {
					DispatchError::Other(UNKNOWN_IDENTIFIER).into()
				}
			}

			fn finished(
				identifier: Self::Identifier,
				final_state: Option<State>,
			) -> DispatchResult {
				if identifier == GLOBAL_IDENTIFIER {
					STATE.with(|cell| {
						*cell.borrow_mut() =
							Some(RunnerState::Finished(final_state.expect("some state")))
					});
					Ok(())
				} else {
					DispatchError::Other(UNKNOWN_IDENTIFIER).into()
				}
			}

			fn remove(identifier: Self::Identifier) -> DispatchResult {
				if identifier == GLOBAL_IDENTIFIER {
					STATE.with(|cell| *cell.borrow_mut() = None);
					Ok(())
				} else {
					DispatchError::Other(UNKNOWN_IDENTIFIER).into()
				}
			}

			fn get_state(identifier: Self::Identifier) -> Option<RunnerState> {
				if identifier == GLOBAL_IDENTIFIER {
					STATE.with(|cell| (*cell.borrow()).clone())
				} else {
					None
				}
			}
		}
	};
}
