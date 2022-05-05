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

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	log,
	sp_runtime::traits::{Dispatchable, Hash, TrailingZeroInput},
	traits::{
		schedule::{DispatchTime, Named},
		LockIdentifier, Randomness,
	},
};
use scale_info::TypeInfo;

use sp_std::prelude::*;

use log::info;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// importing the `weights.rs` here
//pub mod weights;

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod connectfour;
use connectfour::Logic;

const CONNECTFOUR_ID: LockIdentifier = *b"connect4";

/// A type alias for the balance type from this pallet's point of view.
//type BalanceOf<T> = <T as pallet_balances::Config>::Balance;
//const MILLICENTS: u32 = 1_000_000_000;

#[derive(Encode, Decode, Clone, PartialEq, MaxEncodedLen, TypeInfo)]
pub enum BoardState<AccountId> {
	None,
	Running,
	Finished(AccountId),
}

impl<AccountId> Default for BoardState<AccountId> {
	fn default() -> Self {
		Self::None
	}
}

/// Connect four board structure containing two players and the board
#[derive(Encode, Decode, Clone, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub struct BoardStruct<Hash, AccountId, BlockNumber, BoardState> {
	id: Hash,
	red: AccountId,
	blue: AccountId,
	board: [[u8; 6]; 7],
	last_turn: BlockNumber,
	next_player: u8,
	board_state: BoardState,
}

const PLAYER_1: u8 = 1;
const PLAYER_2: u8 = 2;
const MAX_GAMES_PER_BLOCK: u8 = 10;
const MAX_BLOCKS_PER_TURN: u8 = 10;
const CLEANUP_BOARDS_AFTER: u8 = 20;

#[frame_support::pallet]
pub mod pallet {
	use ajuna_common::MatchMaker;
	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::TrailingZeroInput,
	};
	use frame_system::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Proposal: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The generator used to supply randomness to contracts through `seal_random`.
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		type Scheduler: Named<Self::BlockNumber, Self::Proposal, Self::PalletsOrigin>;

		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

		type MatchMaker: MatchMaker<Player = Self::AccountId>;

		// /// Weight information for extrinsics in this pallet.
		//type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn founder_key)]
	pub type FounderKey<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn boards)]
	/// Store all boards that are currently being played.
	pub type Boards<T: Config> = StorageMap<
		_,
		Identity,
		T::Hash,
		BoardStruct<T::Hash, T::AccountId, T::BlockNumber, BoardState<T::AccountId>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn player_board)]
	/// Store players active board, currently only one board per player allowed.
	pub type PlayerBoard<T: Config> = StorageMap<_, Identity, T::AccountId, T::Hash, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn board_schedules)]
	/// Store boards open schedules.
	pub type BoardSchedules<T: Config> =
		StorageMap<_, Identity, T::Hash, Option<Vec<u8>>, ValueQuery>;

	// Default value for Nonce
	#[pallet::type_value]
	pub fn NonceDefault() -> u64 {
		0
	}
	// Nonce used for generating a different seed each time.
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery, NonceDefault>;

	// Used for generating a zeroed account id, copy pasted for now
	struct DefaultAccountIdGenerator<T: Config>(pub T::AccountId);

	impl<T: Config> Default for DefaultAccountIdGenerator<T> {
		fn default() -> Self {
			// Stolen from https://github.com/paritytech/substrate/commit/f57c6447af83a1706041d462ca290b4f2a1bac4f#diff-68096a50d12854e07693a4828590517bb81fea37a9253640278ecdc5b93b6992R860
			let zero_account_id = T::AccountId::decode(&mut TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed");

			DefaultAccountIdGenerator(zero_account_id)
		}
	}

	// The genesis config type.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub founder_key: T::AccountId,
	}

	// The default value for the genesis config type.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { founder_key: DefaultAccountIdGenerator::<T>::default().0 }
		}
	}

	// The build of genesis for the pallet.
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<FounderKey<T>>::put(&self.founder_key);
		}
	}

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		/// A new board got created.
		NewBoard(T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Something went wrong during generating
		BadMetadata,
		/// Couldn't put off a scheduler task as planned.
		ScheduleError,
		/// Player already has a board which is being played.
		PlayerBoardExists,
		/// Player board doesn't exist for this player.
		NoPlayerBoard,
		/// Player can't play against them self.
		NoFakePlay,
		/// Wrong player for next turn.
		NotPlayerTurn,
		/// There was an error while trying to execute something in the logic mod.
		WrongLogic,
		/// Unable to queue, make sure you're not already queued.
		AlreadyQueued,
		/// Extrinsic is limited to founder.
		OnlyFounderAllowed,
		/// Board doesn't exist for this id.
		NoBoardFound,
	}

	// Pallet implements [`Hooks`] trait to define some logic to execute in some context.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// `on_initialize` is executed at the beginning of the block before any extrinsic are
		// dispatched.
		//
		// This function must return the weight consumed by `on_initialize` and `on_finalize`.
		fn on_initialize(_: T::BlockNumber) -> Weight {
			// Anything that needs to be done at the start of the block.
			// We don't do anything here.

			// initial weights
			let mut tot_weights = 10_000;
			for _i in 0..MAX_GAMES_PER_BLOCK {
				// try to create a match till we reached max games or no more matches available
				if let Some(result) = T::MatchMaker::try_match(0, 2) {
					// Create new game
					let _game_id = Self::create_game(result[0].clone(), result[1].clone());
					// weights need to be adjusted
					tot_weights += T::DbWeight::get().reads_writes(1, 1);
					continue
				}
				break
			}

			// return standard weight for trying to find a match
			tot_weights
		}

		// `on_finalize` is executed at the end of block after all extrinsic are dispatched.
		fn on_finalize(_n: BlockNumberFor<T>) {
			// Perform necessary data/state clean up here.
		}

		// A runtime code run after every block and have access to extended set of APIs.
		//
		// For instance you can generate extrinsics for the upcoming produced block.
		fn offchain_worker(_n: T::BlockNumber) {
			// We don't do anything here.
			// but we could dispatch extrinsic (transaction/unsigned/inherent) using
			// sp_io::submit_extrinsic.
			// To see example on offchain worker, please refer to example-offchain-worker pallet
			// accompanied in this repository.
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Print out log or debug message in the console via log::{error, warn, info, debug,
			// trace}, accepting format strings similar to `println!`.
			// https://substrate.dev/rustdocs/v3.0.0/log/index.html
			info!("New value is now: {:?}", something);

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		/// Queue sender up for a game, ranking brackets
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn queue(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// Make sure player has no board open.
			ensure!(!PlayerBoard::<T>::contains_key(&sender), Error::<T>::PlayerBoardExists);

			let bracket = 0;
			ensure!(T::MatchMaker::enqueue(sender, bracket), Error::<T>::AlreadyQueued);

			Ok(())
		}

		/// Empty all brackets, this is a founder only extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn empty_queue(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure sender is founder.
			ensure!(sender == Self::founder_key().unwrap(), Error::<T>::OnlyFounderAllowed);

			let bracket: u32 = 0;
			// Empty queues
			T::MatchMaker::clear_queue(bracket);

			Ok(())
		}

		/// Create game for two players
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn new_game(origin: OriginFor<T>, opponent: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Don't allow playing against yourself.
			ensure!(sender != opponent, Error::<T>::NoFakePlay);

			// Don't allow queued player to create a game.
			ensure!(!T::MatchMaker::is_queued(&sender), Error::<T>::AlreadyQueued);
			ensure!(!T::MatchMaker::is_queued(&opponent), Error::<T>::AlreadyQueued);

			// Make sure players have no board open.
			ensure!(!PlayerBoard::<T>::contains_key(&sender), Error::<T>::PlayerBoardExists);
			ensure!(!PlayerBoard::<T>::contains_key(&opponent), Error::<T>::PlayerBoardExists);

			// Create new game
			let _board_id = Self::create_game(sender, opponent);

			Ok(())
		}

		/// Create game for two players
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn play_turn(origin: OriginFor<T>, column: u8) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(column < 8, "Game only allows columns smaller then 8");

			// TODO: should PlayerBoard storage here be optional to avoid two reads?
			ensure!(PlayerBoard::<T>::contains_key(&sender), Error::<T>::NoPlayerBoard);

			let board_id = Self::player_board(&sender);

			// Get board from player.
			ensure!(Boards::<T>::contains_key(&board_id), Error::<T>::NoBoardFound);
			let mut board = Self::boards(&board_id).ok_or(Error::<T>::NoBoardFound)?;

			// Board is still open to play and not finished.
			ensure!(
				board.board_state == BoardState::Running,
				"Board is not running, check if already finished."
			);

			let current_player = board.next_player;
			let current_account;

			// Check if correct player is at turn
			if current_player == PLAYER_1 {
				current_account = board.red.clone();
				board.next_player = PLAYER_2;
			} else if current_player == PLAYER_2 {
				current_account = board.blue.clone();
				board.next_player = PLAYER_1;
			} else {
				return Err(Error::<T>::WrongLogic.into())
			}

			// Make sure current account is at turn.
			ensure!(sender == current_account, Error::<T>::NotPlayerTurn);

			// Check if we can successfully place a stone in that column
			if !Logic::add_stone(&mut board.board, column, current_player) {
				return Err(Error::<T>::WrongLogic.into())
			}

			// Check if the last played stone gave us a winner or board is full
			if Logic::evaluate(board.board, current_player) {
				board.board_state = BoardState::Finished(current_account);
			} else if Logic::full(board.board) {
				board.board_state =
					BoardState::Finished(DefaultAccountIdGenerator::<T>::default().0);
			}

			// get current blocknumber
			let last_turn = <frame_system::Pallet<T>>::block_number();
			board.last_turn = last_turn;

			// Write next board state back into the storage
			<Boards<T>>::insert(board_id, board);

			// Cancel scheduled task
			if BoardSchedules::<T>::contains_key(&board_id) {
				let old_schedule_id = Self::board_schedules(&board_id);
				if let Some(old_schedule_id) = old_schedule_id {
					// cancel scheduled force end turn
					if T::Scheduler::cancel_named(old_schedule_id).is_err() {
						frame_support::print("LOGIC ERROR: test_schedule/schedule_named failed");
					}
				}
			}

			let schedule_id = Self::schedule_end_turn(
				board_id,
				last_turn,
				last_turn + MAX_BLOCKS_PER_TURN.into(),
			);

			//let bounded_name: BoundedVec<u8, u32> =
			//	schedule_id.unwrap().clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

			<BoardSchedules<T>>::insert(board_id, schedule_id);

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn test_schedule(origin: OriginFor<T>, delay: T::BlockNumber) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let now = <frame_system::Pallet<T>>::block_number();

			let index: u32 = 77;
			let when = now + delay;
			if T::Scheduler::schedule_named(
				(CONNECTFOUR_ID, index).encode(),
				DispatchTime::At(when),
				None,
				63,
				frame_system::RawOrigin::Signed(sender).into(),
				Call::do_something { something: index }.into(),
			)
			.is_err()
			{
				frame_support::print("LOGIC ERROR: test_schedule/schedule_named failed");
				return Err(Error::<T>::ScheduleError.into())
			}

			Ok(())
		}

		/// Force end turn after max blocks per turn passed.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn force_end_turn(
			origin: OriginFor<T>,
			board_id: T::Hash,
			last_turn: T::BlockNumber,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Get board from player.
			ensure!(Boards::<T>::contains_key(&board_id), Error::<T>::NoBoardFound);

			if let Some(mut board) = Self::boards(&board_id) {
				ensure!(board.last_turn == last_turn, "There has been a move in between.");

				if board.board_state == BoardState::Running {
					if board.next_player == PLAYER_1 {
						board.board_state = BoardState::Finished(board.blue.clone());
					} else if board.next_player == PLAYER_2 {
						board.board_state = BoardState::Finished(board.red.clone());
					} else {
						return Err(Error::<T>::WrongLogic.into())
					}

					// get current blocknumber
					let last_turn = <frame_system::Pallet<T>>::block_number();
					board.last_turn = last_turn;

					// Write next board state back into the storage
					<Boards<T>>::insert(board_id, board);

					// Execute cleanup task
					let schedule_id = Self::schedule_end_turn(
						board_id,
						last_turn,
						last_turn + CLEANUP_BOARDS_AFTER.into(),
					);

					<BoardSchedules<T>>::insert(board_id, schedule_id);
				} else {
					// do cleanup after final force turn.
					<Boards<T>>::remove(board_id);
					<PlayerBoard<T>>::remove(board.red);
					<PlayerBoard<T>>::remove(board.blue);
					<BoardSchedules<T>>::remove(board_id);
				}

				Ok(())
			} else {
				Err(DispatchError::from(Error::<T>::NoBoardFound))
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Update nonce once used.
	fn encode_and_update_nonce() -> Vec<u8> {
		let nonce = <Nonce<T>>::get();
		<Nonce<T>>::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	/// Generates a random hash out of a seed.
	fn generate_random_hash(phrase: &[u8], sender: T::AccountId) -> T::Hash {
		let (seed, _) = T::Randomness::random(phrase);
		let seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
			.expect("input is padded with zeroes; qed");
		(seed, &sender, Self::encode_and_update_nonce()).using_encoded(T::Hashing::hash)
	}

	/// Generate a new game between two players.
	fn create_game(red: T::AccountId, blue: T::AccountId) -> T::Hash {
		// get a random hash as board id
		let board_id = Self::generate_random_hash(b"create", red.clone());

		// calculate plyer to start the first turn, with the first byte of the board_id random hash
		let next_player = if board_id.as_ref()[0] < 128 { PLAYER_1 } else { PLAYER_2 };

		// get current blocknumber
		let block_number = <frame_system::Pallet<T>>::block_number();

		// create a new empty game
		let board = BoardStruct {
			id: board_id,
			red: red.clone(),
			blue: blue.clone(),
			board: [[0u8; 6]; 7],
			last_turn: block_number,
			next_player,
			board_state: BoardState::Running,
		};

		// insert the new board into the storage
		<Boards<T>>::insert(board_id, board);

		// Add board to the players playing it.
		<PlayerBoard<T>>::insert(red, board_id);
		<PlayerBoard<T>>::insert(blue, board_id);

		// emit event for a new board creation
		// Emit an event.
		Self::deposit_event(Event::NewBoard(board_id));

		board_id
	}

	/// Schedule end turn
	fn schedule_end_turn(
		board_id: T::Hash,
		last_turn: T::BlockNumber,
		end_turn: T::BlockNumber,
	) -> Option<Vec<u8>> {
		//ensure!(end_turn > <frame_system::Pallet<T>>::block_number(), "Can't schedule a end turn
		// in the past.");
		let schedule_task_id = (CONNECTFOUR_ID, board_id, last_turn).encode();

		if T::Scheduler::schedule_named(
			schedule_task_id.clone(),
			DispatchTime::At(end_turn),
			None,
			63,
			frame_system::RawOrigin::Root.into(),
			Call::force_end_turn { board_id, last_turn }.into(),
		)
		.is_err()
		{
			frame_support::print("LOGIC ERROR: test_schedule/schedule_named failed");
			return None
		}

		Some(schedule_task_id)
	}
}
