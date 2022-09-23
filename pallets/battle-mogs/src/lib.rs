/*
  _______ __                       _______         __      
 |   _   |__|.--.--.-----.---.-.  |    |  |.-----.|  |_.
 |       |  ||  |  |     |  _  |  |       ||  -__||   _|.--.
 |___|___|  ||_____|__|__|___._|  |__|____||_____||____||__|
        |___|                             
  .............<-::]] Ajuna Network (ajuna.io) [[::->.............             
 +-----------------------------------------------------------------
 | This file is part of the BattleMogs project from Ajuna Network.
 ¦-----------------------------------------------------------------
 | Copyright (c) 2022 BloGa Tech AG
 | Copyright (c) 2020 DOT Mog Team (darkfriend77 & metastar77)
 ¦-----------------------------------------------------------------
 | Authors: darkfriend77
 | License: GNU Affero General Public License v3.0
 +-----------------------------------------------------------------
 */
#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

use frame_support::{
	ensure,
	codec::{Encode, Decode},
	traits::{
		Randomness, Currency, ReservableCurrency, ExistenceRequirement, WithdrawReasons
	},
};

use sp_runtime::{DispatchResult, SaturatedConversion, traits::{Hash, TrailingZeroInput, Zero}};
use sp_std::vec::{Vec};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Implementations of some helper traits passed into runtime modules as associated types.
mod types;
pub use types::*;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Something that provides randomness in the runtime.
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn founder_key)]
	/// The `AccountId` of the dot mog founder.
	pub type FounderKey<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account_config)]
	/// A map of the current configuration of an account.
	pub type AccountConfig<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, [u8; 10], OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn mogwai)]
	/// A map of mogwais accessible by the mogwai hash.
	pub type Mogwais<T: Config> = StorageMap<_, Identity, T::Hash, MogwaiStruct<T::Hash, T::BlockNumber, BalanceOf<T>, RarityType, PhaseType>, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn mogwai_prices)]
	/// A map of mogwais that are up for sale.
	pub type MogwaiPrices<T: Config> = StorageMap<_, Identity, T::Hash, BalanceOf<T>, OptionQuery>;
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	/// A map of mogwai owners accessible by the mogwai hash.
	pub type MogwaiOwner<T: Config> = StorageMap<_, Identity, T::Hash, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn mogwai_by_index)]
	/// A map of all existing mogwais accessible by the index.
	pub type AllMogwaisArray<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::Hash, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn all_mogwais_count)]
	/// A count over all existing mogwais in the system.
	pub type AllMogwaisCount<T: Config> = StorageValue<_, u64, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn all_mogwais_hash)]
	/// A map of the index of the mogwai accessible by the mogwai hash.
	pub type AllMogwaisIndex<T: Config> = StorageMap<_, Identity, T::Hash, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn mogwai_of_owner_by_index)]
	/// A map of all mogwai hashes associated with an account.
	pub type OwnedMogwaisArray<T: Config> = StorageMap<_, Blake2_128Concat, (T::AccountId, u64), T::Hash, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn owned_mogwais_count)]
	/// A count over all existing mogwais owned by one account.
	pub type OwnedMogwaisCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn owned_mogwais_hash)]
	/// A map of the owned mogwais index accessible by the mogwai hash.
	pub type OwnedMogwaisIndex<T: Config> = StorageMap<_, Identity, T::Hash, u64, ValueQuery>;

	// Default value for Nonce
	#[pallet::type_value]
	pub fn NonceDefault<T: Config>() -> u64 { 0 }
	// Nonce used for generating a different seed each time.
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery, NonceDefault<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		// A account configuration has been changed.
		AccountConfigChanged(T::AccountId, [u8; GameConfig::PARAM_COUNT]),

		/// A price has been set for a mogwai.
		ForSale(T::AccountId, T::Hash, BalanceOf<T>),

		/// A price has been unset for a mogwai.
		NotForSale(T::AccountId, T::Hash),

		/// A mogwai has been created.
		MogwaiCreated(T::AccountId, T::Hash),

		/// A mogwai changed his owner.
		MogwaiTransfered(T::AccountId, T::AccountId, T::Hash),

		/// A mogwai has been removed. (R.I.P.)
		MogwaiRemoved(T::AccountId, T::Hash),

		/// A mogwai has been was bought.
		MogwaiBought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),

		/// A mogwai has been morphed.
		MogwaiMorphed(T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {

		/// Error names should be descriptive.
		NoneValue,

		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		/// The submitted index is out of range.
		ConfigIndexOutOfRange,

		/// Invalid or unimplemented config update.
		ConfigUpdateInvalid,

		/// Founder action only.
		FounderAction,

		/// Mogwai is not for sale
		MogwaiNotForSale,

		/// The mogwais hash doesn't exist.
		MogwaiDoesntExists,

		/// Maximum Mogwais in account reached.
		MaxMogwaisInAccount,

		/// The mogwai id (hash) already exists.
		MogwaiAlreadyExists,

		/// The mogwai isn't owned by the sender.
		MogwaiNotOwned,

		/// Same mogwai choosen for extrinsic.
		MogwaiSame,

		// Can't hatch mogwai.
		MogwaiNoHatch,

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
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

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
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		/// Update configuration of this sender
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn update_config(origin: OriginFor<T>, index: u8, value_opt: Option<u8>) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(usize::from(index) < GameConfig::PARAM_COUNT, Error::<T>::ConfigIndexOutOfRange);

			let config_opt = <AccountConfig<T>>::get(&sender);
			let mut game_config = GameConfig::new();
			if config_opt.is_some() {
				game_config.parameters = config_opt.unwrap();
			}

			// TODO: add rules (min, max) for different configurations
			let update_value:u8 = GameConfig::verify_update(index, game_config.parameters[usize::from(index)], value_opt);

			ensure!(update_value > 0, Error::<T>::ConfigUpdateInvalid);

			let price = Pricing::config_update_price(index, update_value);
			if price > 0 {
				Self::pay_founder(sender.clone(), price.saturated_into())?;
			}

			game_config.parameters[usize::from(index)] = update_value;

			// updating to the new configuration
			<AccountConfig<T>>::insert(&sender, &game_config.parameters);

			// Emit an event.
			Self::deposit_event(Event::AccountConfigChanged(sender, game_config.parameters));

			// Return a successful DispatchResult
			Ok(())
		}

		/// Set price of mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn set_price(origin: OriginFor<T>, mogwai_id: T::Hash, new_price: BalanceOf<T>) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiNotOwned)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			<MogwaiPrices<T>>::insert(mogwai_id, new_price);
			Self::deposit_event(Event::ForSale(sender, mogwai_id, new_price));

			Ok(())
		}

		/// unset price of mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn remove_price(origin: OriginFor<T>, mogwai_id: T::Hash) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiNotOwned)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);
			ensure!(MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiNotForSale);

			<MogwaiPrices<T>>::remove(mogwai_id);
			Self::deposit_event(Event::NotForSale(sender, mogwai_id));

			Ok(())
		}

		/// Create a new mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn create_mogwai(origin: OriginFor<T>) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			let random_hash = Self::generate_random_hash(b"create_mogwai", sender.clone());

			let (rarity, next_gen) = Generation::next_gen(1, RarityType::Common, 1, RarityType::Common, random_hash.as_ref());

			let block_number = <frame_system::Pallet<T>>::block_number();
			let breed_type : BreedType = Self::calculate_breedtype(block_number);

			let mut dx: [u8;16] = Default::default();
			let mut dy: [u8;16] = Default::default();

			dx.copy_from_slice(&random_hash.as_ref()[0..16]);
			dy.copy_from_slice(&random_hash.as_ref()[16..32]);

			let final_dna : [u8;32] = Breeding::pairing(breed_type, dx, dy);
			// don't know a better way, then using a clone.
			let mut final_dna_hash = random_hash.clone();
			for i in 0..32 {
				final_dna_hash.as_mut()[i] = final_dna[i];
			}

			let new_mogwai = MogwaiStruct {
				id: random_hash,
				dna: final_dna_hash,
				metaxy: [[0u8;16];2],
				genesis: block_number,
				intrinsic: Zero::zero(),
				gen: next_gen,
				rarity: rarity,
				phase: PhaseType::Breeded,
			};

			Self::mint(sender, random_hash, new_mogwai)?;

			Ok(())
		}

		/// Remove an old mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn remove_mogwai(origin: OriginFor<T>, mogwai_id: T::Hash) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::founder_key().unwrap(), Error::<T>::FounderAction);

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiNotOwned)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			Self::remove(sender, mogwai_id)?;

			Ok(())
		}

		/// Transfer mogwai to a new account.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, mogwai_id: T::Hash) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::founder_key().unwrap(), Error::<T>::FounderAction);

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(to.clone()), Error::<T>::MaxMogwaisInAccount);

			Self::transfer_from(sender, to, mogwai_id)?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn hatch_mogwai(origin: OriginFor<T>, mogwai_id: T::Hash) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			let mut mogwai = Self::mogwai(mogwai_id);

			let block_number = <frame_system::Pallet<T>>::block_number();

			ensure!(block_number - mogwai.genesis >= GameEventType::time_till(GameEventType::Hatch).into(), Error::<T>::MogwaiNoHatch);

			let block_hash = <frame_system::Pallet<T>>::block_hash(block_number);

			let metaxy = Self::segment(mogwai.clone(), block_hash);

			mogwai.phase = PhaseType::Hatched;
			mogwai.metaxy = metaxy;

			<Mogwais<T>>::insert(mogwai_id, mogwai);

			Ok(())
		}

		/// Sacrifice mogwai to an other mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn sacrifice(origin: OriginFor<T>, mogwai_id_1: T::Hash) -> DispatchResult {

            let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			// TODO this needs to be check, reworked and corrected, add dynasty feature !!!
			let mogwai_1 = Self::mogwai(mogwai_id_1);

			let intrinsic = mogwai_1.intrinsic / Pricing::intrinsic_return(mogwai_1.phase).saturated_into();
			Self::remove(sender.clone(), mogwai_id_1)?;
			let _ = T::Currency::deposit_into_existing(&sender, intrinsic)?;

            Ok(())
		}

		/// Sacrifice mogwai to an other mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn sacrifice_into(origin: OriginFor<T>, mogwai_id_1: T::Hash, mogwai_id_2: T::Hash) -> DispatchResult {

            let sender = ensure_signed(origin)?;

			let owner1 = Self::owner_of(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			let owner2 = Self::owner_of(mogwai_id_2).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner1 == owner2, Error::<T>::MogwaiNotOwned);
			ensure!(owner1 == sender, Error::<T>::MogwaiNotOwned);

			// asacrificing into the same mogwai isn't allowed
			ensure!(mogwai_id_1 != mogwai_id_2, Error::<T>::MogwaiSame);

			// TODO this needs to be check, reworked and corrected, add dynasty feature !!!
			let mogwai_1 = Self::mogwai(mogwai_id_1);
			let mut mogwai_2 = Self::mogwai(mogwai_id_2);

			ensure!((mogwai_1.rarity as u8 * mogwai_2.rarity as u8) > 0, "Sacrifice into is only available for normal and higher rarity!");

			let gen_jump = Breeding::sacrifice(mogwai_1.gen, mogwai_1.rarity as u32, mogwai_1.metaxy.clone(), mogwai_2.gen, mogwai_2.rarity as u32, mogwai_2.metaxy.clone());
			if gen_jump > 0 && (mogwai_2.gen + gen_jump) <= 16 {
				if mogwai_1.intrinsic > Zero::zero() {
					mogwai_2.intrinsic += mogwai_1.intrinsic; // TODO check overflow
				}
				mogwai_2.gen += gen_jump;
				<Mogwais<T>>::insert(mogwai_id_2, mogwai_2);
			}

			Self::remove(sender.clone(), mogwai_id_1)?;

            Ok(())
		}

		/// Buy a mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn buy_mogwai(origin: OriginFor<T>, mogwai_id: T::Hash, max_price: BalanceOf<T>) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiNotForSale);

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner != sender, "You already own this mogwai");

			let mogwai_price = Self::mogwai_prices(mogwai_id).unwrap();

			ensure!(mogwai_price <= max_price, "You can't buy this mogwai, price exceeds your max price limit");

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			T::Currency::transfer(&sender, &owner, mogwai_price, ExistenceRequirement::KeepAlive)?;

			// Transfer the mogwai using `transfer_from()` including a proof of why it cannot fail
			Self::transfer_from(owner.clone(), sender.clone(), mogwai_id)
				.expect("`owner` is shown to own the mogwai; \
				`owner` must have greater than 0 mogwai, so transfer cannot cause underflow; \
				`all_mogwai_count` shares the same type as `owned_mogwai_count` \
				and minting ensure there won't ever be more than `max()` mogwais, \
				which means transfer cannot cause an overflow; \
				qed");

			Self::deposit_event(Event::MogwaiBought(sender, owner, mogwai_id, mogwai_price));

			Ok(())
		}

		/// Morph a mogwai
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(5,1))]
		pub fn morph_mogwai(origin: OriginFor<T>, mogwai_id: T::Hash) -> DispatchResult {

            let sender = ensure_signed(origin)?;

			// be sure there is such a mogwai.
            ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			// check that the mogwai has an owner and that it is the one calling this extrinsic
			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;
			ensure!(owner == sender, "You don't own this mogwai");

			let mut mogwai = Self::mogwai(mogwai_id);

			let pairing_price:BalanceOf<T> = Pricing::pairing(mogwai.rarity, mogwai.rarity).saturated_into();

			Self::tip_mogwai(sender.clone(), pairing_price, mogwai_id, mogwai.clone())?;

			// get blocknumber to calculate moon phase for morphing
			let block_number = <frame_system::Pallet<T>>::block_number();
			let breed_type : BreedType = Self::calculate_breedtype(block_number);

			let mut dx: [u8;8] = Default::default();
			let mut dy: [u8;8] = Default::default();
			dx.copy_from_slice(&mogwai.dna.as_ref()[0..8]);
			dy.copy_from_slice(&mogwai.dna.as_ref()[8..16]);

			mogwai.metaxy[0] = Breeding::morph(breed_type, dx, dy);

            <Mogwais<T>>::insert(mogwai_id, mogwai);

			Self::deposit_event(Event::MogwaiMorphed(mogwai_id));

			Ok(())
		}

		/// Breed a mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn breed_mogwai(origin: OriginFor<T>, mogwai_id_1: T::Hash, mogwai_id_2: T::Hash) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id_1), Error::<T>::MogwaiDoesntExists);
			ensure!(Mogwais::<T>::contains_key(mogwai_id_2), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id_1).ok_or(Error::<T>::MogwaiNotOwned)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			// breeding into the same mogwai isn't allowed
			ensure!(mogwai_id_1 != mogwai_id_2, Error::<T>::MogwaiSame);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			let mogwai_1 = Self::mogwai(mogwai_id_1);
			let mogwai_2 = Self::mogwai(mogwai_id_2);

			let parents = [mogwai_1.clone(), mogwai_2.clone()];

			let mogwai_id = Self::generate_random_hash(b"breed_mogwai", sender.clone());

			let (rarity, next_gen) = Generation::next_gen(parents[0].gen, parents[0].rarity, parents[1].gen, parents[1].rarity, mogwai_id.as_ref());

			let block_number = <frame_system::Pallet<T>>::block_number();
			let breed_type : BreedType = Self::calculate_breedtype(block_number);

			let dx = mogwai_1.metaxy[0];
			let dy = mogwai_2.metaxy[0];

			// add pairing price to mogwai intrinsic value TODO
			let pairing_price:BalanceOf<T> = Pricing::pairing(parents[0].rarity, parents[1].rarity).saturated_into();
			Self::tip_mogwai(sender.clone(), pairing_price, mogwai_id_2, mogwai_2)?;

			let final_dna : [u8;32] = Breeding::pairing(breed_type, dx, dy);

			// don't know a better way, then using a clone.
			let mut final_dna_hash = mogwai_id.clone();
			for i in 0..32 {
				final_dna_hash.as_mut()[i] = final_dna[i];
			}

			let mogwai_struct = MogwaiStruct {
				id: mogwai_id,
				dna: final_dna_hash,
				metaxy: [[0u8;16];2],
				genesis: block_number,
				intrinsic: Zero::zero(),
				gen: next_gen,
				rarity: rarity,
				phase: PhaseType::Breeded,
			};

			// mint mogwai
			Self::mint(sender, mogwai_id, mogwai_struct)?;

			Ok(())
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

	///
	fn generate_random_hash(phrase: &[u8], sender: T::AccountId) -> T::Hash {

		let (seed, _) = T::Randomness::random(phrase);
		let seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
			 .expect("input is padded with zeroes; qed");
		return (seed, &sender, Self::encode_and_update_nonce()).using_encoded(T::Hashing::hash);
	}

	/// pay fee
	fn pay_fee(who: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {

		let _ =  T::Currency::withdraw(
			&who,
			amount,
			WithdrawReasons::FEE,
			ExistenceRequirement::KeepAlive,
		)?;

		Ok(())
	}

	/// pay founder
	fn pay_founder(who: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {

		let founder: T::AccountId = Self::founder_key().unwrap();

		let _ =  T::Currency::transfer(
			&who,
			&founder,
			amount,
			ExistenceRequirement::KeepAlive,
		)?;

		Ok(())
	}

	/// tiping mogwai
	fn tip_mogwai(who: T::AccountId, amount: BalanceOf<T>, mogwai_id: T::Hash, mut mogwai:  MogwaiStruct<T::Hash, T::BlockNumber, BalanceOf<T>, RarityType, PhaseType> ) -> DispatchResult {

		Self::pay_fee(who, amount)?;

		mogwai.intrinsic += amount; // check for overflow
		<Mogwais<T>>::insert(mogwai_id, mogwai);

		Ok(())
	}

	///
	fn config_value(who: T::AccountId, index: u8) -> u32 {

		let config_opt = <AccountConfig<T>>::get(&who);
		let result:u32;
		if config_opt.is_some() {
			let value = config_opt.unwrap()[usize::from(index)];
			result = GameConfig::config_value(index, value);
		} else {
			result = GameConfig::config_value(index, 0);
		}
		result
	}

	///
	fn ensure_not_max_mogwais(who: T::AccountId) -> bool {
		Self::owned_mogwais_count(&who) < Self::config_value(who.clone(), 1) as u64
	}

	///
	fn mint(to: T::AccountId, mogwai_id: T::Hash, new_mogwai: MogwaiStruct<T::Hash, T::BlockNumber, BalanceOf<T>, RarityType, PhaseType>) -> DispatchResult {

		ensure!(!MogwaiOwner::<T>::contains_key(&mogwai_id), Error::<T>::MogwaiAlreadyExists);

		let owned_mogwais_count = Self::owned_mogwais_count(&to);
		let new_owned_mogwais_count = owned_mogwais_count.checked_add(1)
			.ok_or("Overflow adding a new mogwai to account balance")?;

		let all_mogwais_count = Self::all_mogwais_count();
		let new_all_mogwais_count = all_mogwais_count.checked_add(1)
			.ok_or("Overflow adding a new mogwai to total supply")?;

		// Update maps.
		<Mogwais<T>>::insert(mogwai_id, new_mogwai);
		<MogwaiOwner<T>>::insert(mogwai_id, &to);

		<AllMogwaisArray<T>>::insert(all_mogwais_count, mogwai_id);
        <AllMogwaisCount<T>>::put(new_all_mogwais_count);
        <AllMogwaisIndex<T>>::insert(mogwai_id, all_mogwais_count);

		<OwnedMogwaisArray<T>>::insert((to.clone(), owned_mogwais_count), mogwai_id);
        <OwnedMogwaisCount<T>>::insert(&to, new_owned_mogwais_count);
        <OwnedMogwaisIndex<T>>::insert(mogwai_id, owned_mogwais_count);

		// Emit an event.
		Self::deposit_event(Event::MogwaiCreated(to, mogwai_id));

		Ok(())
	}

	///
	fn remove(from: T::AccountId, mogwai_id: T::Hash) -> DispatchResult {

		ensure!(MogwaiOwner::<T>::contains_key(&mogwai_id), Error::<T>::MogwaiDoesntExists);

		let owned_mogwais_count = Self::owned_mogwais_count(&from);
		let new_owned_mogwai_count = owned_mogwais_count.checked_sub(1)
			.ok_or("Overflow removing an old mogwai from account balance")?;

		let all_mogwais_count = Self::all_mogwais_count();
		let new_all_mogwais_count = all_mogwais_count.checked_sub(1)
			.ok_or("Overflow removing an old mogwai to total supply")?;

		// Update maps.
		<Mogwais<T>>::remove(mogwai_id);
		<MogwaiOwner<T>>::remove(mogwai_id);

		// remove storage entries
		if MogwaiPrices::<T>::contains_key(mogwai_id) {
			MogwaiPrices::<T>::remove(mogwai_id);
		}

        let all_mogwai_index = <AllMogwaisIndex<T>>::get(mogwai_id);
        if all_mogwai_index != new_all_mogwais_count {
            let all_last_mogwai_id = <AllMogwaisArray<T>>::get(new_all_mogwais_count);
            <AllMogwaisArray<T>>::insert(all_mogwai_index, all_last_mogwai_id);
            <AllMogwaisIndex<T>>::insert(all_last_mogwai_id, all_mogwai_index);
		}

		<AllMogwaisArray<T>>::remove(new_all_mogwais_count);
		<AllMogwaisCount<T>>::put(new_all_mogwais_count);
		<AllMogwaisIndex<T>>::remove(mogwai_id);

        let mogwai_index = <OwnedMogwaisIndex<T>>::get(mogwai_id);
        if mogwai_index != new_owned_mogwai_count {
            let last_mogwai_id = <OwnedMogwaisArray<T>>::get((from.clone(), new_owned_mogwai_count));
            <OwnedMogwaisArray<T>>::insert((from.clone(), mogwai_index), last_mogwai_id);
            <OwnedMogwaisIndex<T>>::insert(last_mogwai_id, mogwai_index);
		}

		<OwnedMogwaisArray<T>>::remove((from.clone(), new_owned_mogwai_count));
		<OwnedMogwaisCount<T>>::insert(&from, new_owned_mogwai_count);
		<OwnedMogwaisIndex<T>>::remove(mogwai_id);

		// Emit an event.
		Self::deposit_event(Event::MogwaiRemoved(from, mogwai_id));

		Ok(())
	}

	/// transfer from
	fn transfer_from(from: T::AccountId, to: T::AccountId, mogwai_id: T::Hash) -> DispatchResult {

		let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;

        ensure!(owner == from, "You don't own this mogwai");

        let owned_mogwai_count_from = Self::owned_mogwais_count(&from);
        let owned_mogwai_count_to = Self::owned_mogwais_count(&to);

		let new_owned_mogwai_count_from = owned_mogwai_count_from.checked_sub(1)
			.ok_or("Overflow removing a mogwai from account")?;
		let new_owned_mogwai_count_to = owned_mogwai_count_to.checked_add(1)
			.ok_or("Overflow adding a mogwai to account")?;

        // NOTE: This is the "swap and pop" algorithm we have added for you
        //       We use our storage items to help simplify the removal of elements from the OwnedMogwaisArray
        //       We switch the last element of OwnedMogwaisArray with the element we want to remove
        let mogwai_index = <OwnedMogwaisIndex<T>>::get(mogwai_id);
        if mogwai_index != new_owned_mogwai_count_from {
            let last_mogwai_id = <OwnedMogwaisArray<T>>::get((from.clone(), new_owned_mogwai_count_from));
            <OwnedMogwaisArray<T>>::insert((from.clone(), mogwai_index), last_mogwai_id);
            <OwnedMogwaisIndex<T>>::insert(last_mogwai_id, mogwai_index);
        }

		// Now we can remove this item by removing the last element
		<MogwaiOwner<T>>::insert(mogwai_id, &to);
		<OwnedMogwaisIndex<T>>::insert(mogwai_id, owned_mogwai_count_to);

		<OwnedMogwaisArray<T>>::remove((from.clone(), new_owned_mogwai_count_from));
		<OwnedMogwaisArray<T>>::insert((to.clone(), owned_mogwai_count_to), mogwai_id);

		// Update the OwnedMogwaisCount for `from` and `to`
		<OwnedMogwaisCount<T>>::insert(&from, new_owned_mogwai_count_from);
		<OwnedMogwaisCount<T>>::insert(&to, new_owned_mogwai_count_to);

		// remove storage entries
		if MogwaiPrices::<T>::contains_key(mogwai_id) {
			MogwaiPrices::<T>::remove(mogwai_id);
		}

		// Emit an event.
		Self::deposit_event(Event::MogwaiTransfered(from, to, mogwai_id));

        Ok(())
	}

	/// Calculate breed type
	fn calculate_breedtype(block_number: T::BlockNumber) -> BreedType {

		let mod_value : u32 = 80;
		let modulo80 = (block_number % mod_value.into()).saturated_into::<u32>();
		if modulo80 < 20 {
			return BreedType::DomDom;
		} else if modulo80 < 40 {
			return BreedType::DomRez;
		} else if modulo80 < 60 {
			return BreedType::RezDom;
		} else {
			return BreedType::RezRez;
		}
	}

	/// do the segmentation
	fn segment(mogwai_struct: MogwaiStruct<T::Hash, T::BlockNumber, BalanceOf<T>, RarityType, PhaseType>, block_hash: T::Hash) -> [[u8;16];2] {

		let mut dna: [u8; 32] = Default::default();
		let mut blk: [u8; 32] = Default::default();

		dna.copy_from_slice(&mogwai_struct.dna.as_ref()[0..32]);
		blk.copy_from_slice(&block_hash.as_ref()[0..32]);

		// segmenting the hatched mogwai
		let (dna,evo) = Breeding::segmenting(dna,blk);

		let mut metaxy = [[0u8;16];2];
		metaxy[0] = dna;
		metaxy[1] = evo;

		metaxy
	}

}
