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
	codec::{Decode, Encode},
	ensure,
	traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency, WithdrawReasons},
};

use sp_runtime::{
	traits::{Hash, Saturating, TrailingZeroInput, Zero},
	DispatchResult, SaturatedConversion,
};
use sp_std::{prelude::*, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod algorithm;
mod types;

pub use algorithm::*;
pub use types::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, BoundedBTreeSet};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{Bounded, Saturating};

	// important to use outside structs and consts
	use super::*;

	pub(crate) type MogwaiIdOf<T> = <T as frame_system::Config>::Hash;
	pub(crate) type MogwaiOf<T> = MogwaiStruct<
		MogwaiIdOf<T>,
		<T as frame_system::Config>::BlockNumber,
		BalanceOf<T>,
		MogwaiGeneration,
		PhaseType,
	>;
	pub(crate) type BoundedMogwaiIdsOf<T> =
		BoundedBTreeSet<MogwaiIdOf<T>, ConstU32<MAX_MOGWAIS_PER_PLAYER>>;
	pub(crate) type MogwaiCount = u64;

	pub(crate) const MAX_MOGWAIS_PER_PLAYER: u32 = 24;

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

	#[pallet::storage]
	#[pallet::getter(fn organizer)]
	pub type Organizer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account_config)]
	/// A map of the current configuration of an account.
	pub type AccountConfig<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, [u8; 10], OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account_achievements)]
	pub type AccountAchievements<T: Config> = StorageDoubleMap<
		_,
		Identity,
		T::AccountId,
		Identity,
		AccountAchievement,
		AchievementState,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn mogwai)]
	/// A map of mogwais accessible by the mogwai hash.
	pub type Mogwais<T: Config> = StorageMap<_, Identity, MogwaiIdOf<T>, MogwaiOf<T>, OptionQuery>;
	#[pallet::storage]
	#[pallet::getter(fn mogwai_prices)]
	/// A map of mogwais that are up for sale.
	pub type MogwaiPrices<T: Config> =
		StorageMap<_, Identity, MogwaiIdOf<T>, BalanceOf<T>, OptionQuery>;
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	/// A map of mogwai owners accessible by the mogwai hash.
	pub type MogwaiOwner<T: Config> =
		StorageMap<_, Identity, MogwaiIdOf<T>, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_mogwais_count)]
	/// A count over all existing mogwais in the system.
	pub type AllMogwaisCount<T: Config> = StorageValue<_, MogwaiCount, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owners)]
	pub type Owners<T: Config> =
		StorageMap<_, Identity, T::AccountId, BoundedMogwaiIdsOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owned_mogwais_count)]
	/// A count over all existing mogwais owned by one account.
	pub type OwnedMogwaisCount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, MogwaiCount, ValueQuery>;

	/// Default value for Nonce
	#[pallet::type_value]
	pub fn NonceDefault<T: Config>() -> u64 {
		0
	}
	/// Nonce used for generating a different seed each time.
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery, NonceDefault<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new organizer has been set.
		OrganizerSet(T::AccountId),

		/// A account configuration has been changed.
		AccountConfigChanged(T::AccountId, [u8; GameConfig::PARAM_COUNT]),

		/// A price has been set for a mogwai.
		ForSale(T::AccountId, T::Hash, BalanceOf<T>),

		/// A price has been unset for a mogwai.
		RemovedFromSale(T::AccountId, T::Hash),

		/// A mogwai was created, by the Emperor himself!.
		MogwaiCreated(T::AccountId, T::Hash),

		/// A mogwai was removed, by the Emperor himself!
		MogwaiRemoved(T::AccountId, T::Hash),

		/// A mogwai was transfered, by the Emperor himself!
		MogwaiTransfered(T::AccountId, T::AccountId, T::Hash),

		/// A mogwai has been bought.
		MogwaiBought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),

		/// A mogwai has been hatched.
		MogwaiHatched(T::AccountId, T::Hash),

		/// A mogwai has been sacrificed.
		MogwaiSacrificed(T::AccountId, T::Hash),

		/// A mogwai has been sacrificed for an other one.
		MogwaiSacrificedInto(T::AccountId, T::Hash, T::Hash),

		/// A mogwai has been morphed.
		MogwaiMorphed(T::Hash),

		/// A mogwai has been bred.
		MogwaiBred(T::Hash),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No organizer set.
		NoOrganizer,

		/// The submitted index is out of range.
		ConfigIndexOutOfRange,

		/// Invalid or unimplemented config update.
		ConfigUpdateInvalid,

		/// Price for config updated not set.
		PriceInvalid,

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

		/// The mogwai hasn't the necessary rarity.
		MogwaiBadRarity,

		/// Same mogwai chosen for extrinsic.
		MogwaiSame,

		// Can't hatch mogwai.
		MogwaiNoHatch,

		/// Can't perform specified action while mogwai is on sale
		MogwaiIsOnSale,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set organizer, this is a sudo call.
		#[pallet::weight(10_000)]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			Organizer::<T>::put(&organizer);

			// Emit an event.
			Self::deposit_event(Event::OrganizerSet(organizer));

			Ok(())
		}

		/// Update configuration of this sender
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn update_config(
			origin: OriginFor<T>,
			index: u8,
			value_opt: Option<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				usize::from(index) < GameConfig::PARAM_COUNT,
				Error::<T>::ConfigIndexOutOfRange
			);

			let config_opt = AccountConfig::<T>::get(&sender);
			let mut game_config = GameConfig::new();
			if config_opt.is_some() {
				game_config.parameters = config_opt.unwrap();
			}

			// TODO: add rules (min, max) for different configurations
			let update_value: u8 = GameConfig::verify_update(
				index,
				game_config.parameters[usize::from(index)],
				value_opt,
			);
			ensure!(update_value > 0, Error::<T>::ConfigUpdateInvalid);

			let price = Pricing::config_update_price(index, update_value);
			ensure!(price > 0, Error::<T>::PriceInvalid);

			let organizer_opt = Self::organizer();
			ensure!(organizer_opt.is_some(), Error::<T>::NoOrganizer);

			Self::pay_founder(sender.clone(), organizer_opt.unwrap(), price.saturated_into())?;

			game_config.parameters[usize::from(index)] = update_value;

			// updating to the new configuration
			AccountConfig::<T>::insert(&sender, &game_config.parameters);

			// Emit an event.
			Self::deposit_event(Event::AccountConfigChanged(sender, game_config.parameters));

			Ok(())
		}

		/// Set price of mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn set_price(
			origin: OriginFor<T>,
			mogwai_id: MogwaiIdOf<T>,
			new_price: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			MogwaiPrices::<T>::insert(mogwai_id, new_price);
			Self::deposit_event(Event::ForSale(sender, mogwai_id, new_price));

			Ok(())
		}

		/// Clear previously set mogwai price.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn remove_price(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);
			ensure!(MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiNotForSale);

			MogwaiPrices::<T>::remove(mogwai_id);
			Self::deposit_event(Event::RemovedFromSale(sender, mogwai_id));

			Ok(())
		}

		/// Create a new mogwai egg.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn create_mogwai(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// TODO Add for production!
			//ensure!(sender == Self::organizer().unwrap(), Error::<T>::FounderAction);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			let random_hash_1 = Self::generate_random_hash(b"create_mogwai", sender.clone());
			let random_hash_2 = Self::generate_random_hash(b"extend_mogwai", sender.clone());

			let (rarity, next_gen, max_rarity) = Generation::next_gen(
				MogwaiGeneration::First,
				RarityType::Common,
				MogwaiGeneration::First,
				RarityType::Common,
				random_hash_1.as_ref(),
			);

			let block_number = <frame_system::Pallet<T>>::block_number();
			let breed_type: BreedType = Self::calculate_breedtype(block_number);

			let dx =
				unsafe { &*(&random_hash_1.as_ref()[0..32] as *const [u8] as *const [u8; 32]) };
			let dy =
				unsafe { &*(&random_hash_2.as_ref()[0..32] as *const [u8] as *const [u8; 32]) };

			let final_dna = Breeding::pairing(breed_type, dx, dy);

			let new_mogwai = MogwaiStruct {
				id: random_hash_1.clone(),
				dna: final_dna,
				genesis: block_number,
				intrinsic: Zero::zero(),
				generation: next_gen,
				rarity: ((max_rarity as u8) << 4) + rarity as u8,
				phase: PhaseType::Breeded,
			};

			Self::mint(&sender, random_hash_1, new_mogwai)?;

			// Emit an event.
			Self::deposit_event(Event::MogwaiCreated(sender, random_hash_1));

			Ok(())
		}

		/// Remove a given mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn remove_mogwai(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::organizer().unwrap(), Error::<T>::FounderAction);

			Self::remove(sender.clone(), mogwai_id)?;

			// Emit an event.
			Self::deposit_event(Event::MogwaiRemoved(sender, mogwai_id));

			Ok(())
		}

		/// Transfer mogwai to another account.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,1))]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			mogwai_id: MogwaiIdOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::organizer().unwrap(), Error::<T>::FounderAction);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(to.clone()), Error::<T>::MaxMogwaisInAccount);

			Self::transfer_from(sender.clone(), to.clone(), mogwai_id)?;

			// Emit an event.
			Self::deposit_event(Event::MogwaiTransfered(sender, to, mogwai_id));

			Ok(())
		}

		/// Hatch a mogwai egg
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn hatch_mogwai(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			let mut mogwai: MogwaiOf<T> =
				Self::mogwai(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;

			let block_number = <frame_system::Pallet<T>>::block_number();

			ensure!(
				block_number - mogwai.genesis >=
					GameEventType::time_till(GameEventType::Hatch).into(),
				Error::<T>::MogwaiNoHatch
			);

			let block_hash = <frame_system::Pallet<T>>::block_hash(block_number);

			let (dna, rarity) = Self::segment_and_bake(mogwai.clone(), block_hash);

			mogwai.phase = PhaseType::Hatched;
			mogwai.rarity = rarity;
			mogwai.dna = dna;

			Mogwais::<T>::insert(mogwai_id, mogwai);

			// TODO: Do something with the result
			let _ = Self::update_achievement_for(&sender, AccountAchievement::EggHatcher, 1);

			// Emit an event.
			Self::deposit_event(Event::MogwaiHatched(sender, mogwai_id));

			Ok(())
		}

		/// Sacrifice mogwai to get some currency
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,1))]
		pub fn sacrifice(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			ensure!(!MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiIsOnSale);

			// TODO this needs to be check, reworked and corrected, add dynasty feature !!!
			let mogwai_1 = Self::mogwai(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;

			ensure!(mogwai_1.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);

			let intrinsic_to_deposit = {
				let computed_intrinsic =
					mogwai_1.intrinsic / Pricing::intrinsic_return(mogwai_1.phase).saturated_into();

				let max_intrinsic =
					BalanceOf::<T>::max_value() - T::Currency::free_balance(&sender);

				std::cmp::min(computed_intrinsic, max_intrinsic)
			};

			Self::remove(sender.clone(), mogwai_id)?;

			// TODO check this function on return value
			let _ = T::Currency::deposit_into_existing(&sender, intrinsic_to_deposit)?;

			// TODO: Do something with the results
			let _ = Self::update_achievement_for(&sender, AccountAchievement::Sacrificer, 1);

			// Emit an event.
			Self::deposit_event(Event::MogwaiSacrificed(sender, mogwai_id));

			Ok(())
		}

		/// Sacrifice mogwai to an other mogwai.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,1))]
		pub fn sacrifice_into(
			origin: OriginFor<T>,
			mogwai_id_1: T::Hash,
			mogwai_id_2: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let owner1 = Self::owner_of(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			let owner2 = Self::owner_of(mogwai_id_2).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!((owner1 == sender) && (owner2 == sender), Error::<T>::MogwaiNotOwned);

			// Sacrificing into the same mogwai isn't allowed
			ensure!(mogwai_id_1 != mogwai_id_2, Error::<T>::MogwaiSame);

			ensure!(!MogwaiPrices::<T>::contains_key(mogwai_id_1), Error::<T>::MogwaiIsOnSale);
			ensure!(!MogwaiPrices::<T>::contains_key(mogwai_id_2), Error::<T>::MogwaiIsOnSale);

			// TODO this needs to be check, reworked and corrected, add dynasty feature !!!
			let mogwai_1: MogwaiOf<T> =
				Self::mogwai(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			let mut mogwai_2: MogwaiOf<T> =
				Self::mogwai(mogwai_id_2).ok_or(Error::<T>::MogwaiDoesntExists)?;

			let mogwai_1_rarity = mogwai_1.rarity as u32;
			let mogwai_2_rarity = mogwai_2.rarity as u32;
			ensure!((mogwai_1_rarity * mogwai_2_rarity) > 0, Error::<T>::MogwaiBadRarity);
			ensure!(mogwai_1.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);
			ensure!(mogwai_2.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);

			let gen_jump = Breeding::sacrifice(
				mogwai_1.generation,
				mogwai_1_rarity,
				&mogwai_1.dna,
				mogwai_2.generation,
				mogwai_2_rarity,
				&mogwai_2.dna,
			) as u16;

			if gen_jump > 0 && (mogwai_2.generation as u16 + gen_jump) <= 16 {
				mogwai_2.intrinsic = mogwai_2.intrinsic.saturating_add(mogwai_1.intrinsic);
				mogwai_2.generation =
					MogwaiGeneration::coerce_from(mogwai_2.generation as u16 + gen_jump as u16);
				Mogwais::<T>::insert(mogwai_id_2, mogwai_2);
			}

			Self::remove(sender.clone(), mogwai_id_1)?;

			// TODO: Do something with the results
			let _ = Self::update_achievement_for(&sender, AccountAchievement::Sacrificer, 1);

			// Emit an event.
			Self::deposit_event(Event::MogwaiSacrificedInto(sender, mogwai_id_1, mogwai_id_2));

			Ok(())
		}

		/// Buy a mogwai on sale
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn buy_mogwai(
			origin: OriginFor<T>,
			mogwai_id: MogwaiIdOf<T>,
			max_price: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiNotForSale);

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner != sender, "You already own this mogwai");

			let mogwai_price = Self::mogwai_prices(mogwai_id).unwrap();

			ensure!(
				mogwai_price <= max_price,
				"You can't buy this mogwai, price exceeds your max price limit"
			);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			T::Currency::transfer(&sender, &owner, mogwai_price, ExistenceRequirement::KeepAlive)?;

			// Transfer the mogwai using `transfer_from()` including a proof of why it cannot fail
			Self::transfer_from(owner.clone(), sender.clone(), mogwai_id).expect(
				"`owner` is shown to own the mogwai; \
				`owner` must have greater than 0 mogwai, so transfer cannot cause underflow; \
				`all_mogwai_count` shares the same type as `owned_mogwai_count` \
				and minting ensure there won't ever be more than `max()` mogwais, \
				which means transfer cannot cause an overflow; \
				qed",
			);

			// TODO: Do something with the results
			let _ = Self::update_achievement_for(&sender, AccountAchievement::Buyer, 1);
			let _ = Self::update_achievement_for(&owner, AccountAchievement::Seller, 1);

			// Emit an event.
			Self::deposit_event(Event::MogwaiBought(sender, owner, mogwai_id, mogwai_price));

			Ok(())
		}

		/// Morph a mogwai
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(6,1))]
		pub fn morph_mogwai(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// be sure there is such a mogwai.
			ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			ensure!(!MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiIsOnSale);

			// check that the mogwai has an owner and that it is the one calling this extrinsic
			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;
			ensure!(owner == sender, "You don't own this mogwai");

			let mut mogwai: MogwaiOf<T> =
				Self::mogwai(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(mogwai.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);
			let mogwai_rarity = mogwai.rarity.unwrap_or_default();

			let pairing_price: BalanceOf<T> =
				Pricing::pairing(mogwai_rarity, mogwai_rarity).saturated_into();

			Self::tip_mogwai(sender.clone(), pairing_price, mogwai_id, &mut mogwai)?;

			// get blocknumber to calculate moon phase for morphing
			let block_number = <frame_system::Pallet<T>>::block_number();
			let breed_type: BreedType = Self::calculate_breedtype(block_number);

			let dx = unsafe { &*(&mogwai.dna[0][0..16] as *const [u8] as *const [u8; 16]) };
			let dy = unsafe { &*(&mogwai.dna[0][16..32] as *const [u8] as *const [u8; 16]) };

			mogwai.dna[0] = Breeding::morph(breed_type, dx, dy);

			Mogwais::<T>::insert(mogwai_id, mogwai);

			// TODO: Do something with the results
			let _ = Self::update_achievement_for(&sender, AccountAchievement::Morpheus, 1);

			// Emit an event.
			Self::deposit_event(Event::MogwaiMorphed(mogwai_id));

			Ok(())
		}

		/// Breed a mogwai with another
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn breed_mogwai(
			origin: OriginFor<T>,
			mogwai_id_1: T::Hash,
			mogwai_id_2: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id_1), Error::<T>::MogwaiDoesntExists);
			ensure!(Mogwais::<T>::contains_key(mogwai_id_2), Error::<T>::MogwaiDoesntExists);

			let owner_1 = Self::owner_of(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			let owner_2 = Self::owner_of(mogwai_id_2).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner_1 == sender, Error::<T>::MogwaiNotOwned);

			// breeding into the same mogwai isn't allowed
			ensure!(mogwai_id_1 != mogwai_id_2, Error::<T>::MogwaiSame);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			let mogwai_1: MogwaiOf<T> =
				Self::mogwai(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			let mut mogwai_2: MogwaiOf<T> =
				Self::mogwai(mogwai_id_2).ok_or(Error::<T>::MogwaiDoesntExists)?;

			ensure!(mogwai_1.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);
			ensure!(mogwai_2.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);

			let mogwai_1_rarity = mogwai_1.rarity.unwrap_or_default();
			let mogwai_2_rarity = mogwai_2.rarity.unwrap_or_default();

			let parents = [mogwai_1.clone(), mogwai_2.clone()];

			let mogwai_id = Self::generate_random_hash(b"breed_mogwai", sender.clone());

			let (rarity, next_gen, max_rarity) = Generation::next_gen(
				parents[0].generation,
				RarityType::from(mogwai_1.rarity),
				parents[1].generation,
				RarityType::from(mogwai_2.rarity),
				mogwai_id.as_ref(),
			);

			let block_number = <frame_system::Pallet<T>>::block_number();
			let breed_type: BreedType = Self::calculate_breedtype(block_number);

			// add pairing price to mogwai intrinsic value TODO
			let pairing_price: BalanceOf<T> =
				Pricing::pairing(mogwai_1_rarity, mogwai_2_rarity).saturated_into();
			Self::tip_mogwai(sender.clone(), pairing_price, mogwai_id_2, &mut mogwai_2)?;

			let final_dna = Breeding::pairing(breed_type, &mogwai_1.dna[0], &mogwai_2.dna[0]);
			let mogwai_rarity = ((max_rarity as u8) << 4) + rarity as u8;

			let mogwai_struct = MogwaiStruct {
				id: mogwai_id,
				dna: final_dna,
				genesis: block_number,
				intrinsic: Zero::zero(),
				generation: next_gen,
				rarity: mogwai_rarity,
				phase: PhaseType::Breeded,
			};

			// mint mogwai
			Self::mint(&sender, mogwai_id, mogwai_struct)?;

			if mogwai_rarity == 16 {
				// TODO: Do something with the results
				let _ = Self::update_achievement_for(&sender, AccountAchievement::LegendBreeder, 1);
			}

			if owner_1 != owner_2 {
				// TODO: Do something with the results
				let _ = Self::update_achievement_for(&sender, AccountAchievement::Promiscuous, 1);
			}

			// Emit an event.
			Self::deposit_event(Event::MogwaiBred(mogwai_id));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Update nonce once used.
	fn encode_and_update_nonce() -> Vec<u8> {
		Nonce::<T>::mutate(|nonce| {
			*nonce = nonce.wrapping_add(1);
			*nonce
		})
		.encode()
	}

	///
	fn generate_random_hash(phrase: &[u8], sender: T::AccountId) -> T::Hash {
		let (seed, _) = T::Randomness::random(phrase);
		let decoded_seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
			.expect("input is padded with zeroes; qed");
		(decoded_seed, &sender, Self::encode_and_update_nonce()).using_encoded(T::Hashing::hash)
	}

	/// pay fee
	fn pay_fee(who: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
		let _ = T::Currency::withdraw(
			&who,
			amount,
			WithdrawReasons::FEE,
			ExistenceRequirement::KeepAlive,
		)?;

		Ok(())
	}

	/// pay founder
	fn pay_founder(
		who: T::AccountId,
		organizer: T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		let _ = T::Currency::transfer(&who, &organizer, amount, ExistenceRequirement::KeepAlive)?;

		Ok(())
	}

	/// tiping mogwai
	fn tip_mogwai(
		who: T::AccountId,
		amount: BalanceOf<T>,
		mogwai_id: MogwaiIdOf<T>,
		mogwai: &mut MogwaiOf<T>,
	) -> DispatchResult {
		Self::pay_fee(who, amount)?;

		mogwai.intrinsic = mogwai.intrinsic.saturating_add(amount);
		Mogwais::<T>::insert(mogwai_id, mogwai);

		Ok(())
	}

	///
	pub(crate) fn config_value(who: T::AccountId, index: u8) -> u32 {
		let config_opt = AccountConfig::<T>::get(&who);
		let result: u32;
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

	/// Add mogwai to storage
	fn mint(
		to: &T::AccountId,
		mogwai_id: MogwaiIdOf<T>,
		new_mogwai: MogwaiOf<T>,
	) -> DispatchResult {
		ensure!(!MogwaiOwner::<T>::contains_key(&mogwai_id), Error::<T>::MogwaiAlreadyExists);

		let new_owned_mogwais_count = Self::owned_mogwais_count(to)
			.checked_add(1)
			.ok_or("Overflow adding a new mogwai to account balance")?;

		let new_all_mogwais_count = Self::all_mogwais_count()
			.checked_add(1)
			.ok_or("Overflow adding a new mogwai to total supply")?;

		// Update maps.
		Mogwais::<T>::insert(mogwai_id, new_mogwai);

		MogwaiOwner::<T>::insert(mogwai_id, to);
		Owners::<T>::try_mutate(to, |id_set| id_set.try_insert(mogwai_id))
			.map_err(|_| Error::<T>::MaxMogwaisInAccount)?;

		AllMogwaisCount::<T>::put(new_all_mogwais_count);
		OwnedMogwaisCount::<T>::insert(to, new_owned_mogwais_count);

		Ok(())
	}

	///
	fn remove(from: T::AccountId, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
		ensure!(MogwaiOwner::<T>::contains_key(&mogwai_id), Error::<T>::MogwaiDoesntExists);

		let new_owned_mogwai_count = Self::owned_mogwais_count(&from)
			.checked_sub(1)
			.ok_or("Overflow removing an old mogwai from account balance")?;

		let new_all_mogwais_count = Self::all_mogwais_count()
			.checked_sub(1)
			.ok_or("Overflow removing an old mogwai to total supply")?;

		// Update maps.
		Mogwais::<T>::remove(mogwai_id);

		MogwaiOwner::<T>::remove(mogwai_id);
		Owners::<T>::mutate(&from, |id_set| {
			id_set.remove(&mogwai_id);
		});

		// remove storage entries
		if MogwaiPrices::<T>::contains_key(mogwai_id) {
			MogwaiPrices::<T>::remove(mogwai_id);
		}

		AllMogwaisCount::<T>::put(new_all_mogwais_count);
		OwnedMogwaisCount::<T>::insert(&from, new_owned_mogwai_count);

		Ok(())
	}

	/// transfer from
	fn transfer_from(
		from: T::AccountId,
		to: T::AccountId,
		mogwai_id: MogwaiIdOf<T>,
	) -> DispatchResult {
		let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;

		ensure!(owner == from, Error::<T>::MogwaiNotOwned);

		let new_owned_mogwai_count_from = Self::owned_mogwais_count(&from)
			.checked_sub(1)
			.ok_or("Overflow removing a mogwai from account")?;
		let new_owned_mogwai_count_to = Self::owned_mogwais_count(&to)
			.checked_add(1)
			.ok_or("Overflow adding a mogwai to account")?;

		// Now we can remove this item by removing the last element
		MogwaiOwner::<T>::insert(mogwai_id, &to);

		// Update the OwnedMogwaisCount for `from` and `to`
		OwnedMogwaisCount::<T>::insert(&from, new_owned_mogwai_count_from);
		Owners::<T>::mutate(from, |id_set| {
			id_set.remove(&mogwai_id);
		});

		OwnedMogwaisCount::<T>::insert(&to, new_owned_mogwai_count_to);
		Owners::<T>::try_mutate(&to, |id_set| id_set.try_insert(mogwai_id))
			.map_err(|_| Error::<T>::MaxMogwaisInAccount)?;

		// remove storage entries
		if MogwaiPrices::<T>::contains_key(mogwai_id) {
			MogwaiPrices::<T>::remove(mogwai_id);
		}

		Ok(())
	}

	/// Calculate breed type
	fn calculate_breedtype(block_number: T::BlockNumber) -> BreedType {
		let mod_value: u32 = 80;
		let modulo80 = (block_number % mod_value.into()).saturated_into::<u32>();

		match modulo80 {
			0..=19 => BreedType::DomDom,
			20..=39 => BreedType::DomRez,
			40..=59 => BreedType::RezDom,
			_ => BreedType::RezRez,
		}
	}

	/// do the segmentation and baking
	fn segment_and_bake(mogwai: MogwaiOf<T>, block_hash: T::Hash) -> ([[u8; 32]; 2], u8) {
		let mut blk: [u8; 32] = [0u8; 32];

		blk.copy_from_slice(&block_hash.as_ref()[0..32]);

		// segment and and bake the hatched mogwai
		(Breeding::segmenting(mogwai.dna.clone(), blk), Breeding::bake(mogwai.rarity, blk))
	}

	#[inline]
	fn update_achievement_for(
		account: &T::AccountId,
		achievement: AccountAchievement,
		update_amount: u16,
	) -> bool {
		AccountAchievements::<T>::mutate(account, achievement, |maybe_value| match maybe_value {
			None => {
				*maybe_value =
					Some(AchievementState::new(achievement.target_for()).update(update_amount));
				false
			},
			Some(AchievementState::Completed) => false,
			Some(value) => {
				let updated_value = value.update(update_amount);
				*maybe_value = Some(updated_value);
				updated_value == AchievementState::Completed
			},
		})
	}
}
