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

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency, WithdrawReasons},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{Hash, Saturating, TrailingZeroInput, Zero},
	DispatchResult, SaturatedConversion,
};
use sp_std::{mem::MaybeUninit, prelude::*, ptr::copy_nonoverlapping, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod migration;

mod algorithm;
mod types;
pub mod weights;

pub use algorithm::*;
pub use types::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	pub use crate::weights::WeightInfo;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, BoundedBTreeSet};
	use sp_runtime::traits::{Bounded, Saturating};

	use super::*;

	pub(crate) type MogwaiIdOf<T> = <T as frame_system::Config>::Hash;
	pub(crate) type MogwaiOf<T> = MogwaiStruct<
		MogwaiIdOf<T>,
		BlockNumberFor<T>,
		BalanceOf<T>,
		MogwaiGeneration,
		RarityType,
		PhaseType,
		<T as frame_system::Config>::AccountId,
	>;
	pub(crate) type BoundedMogwaiIdsOf<T> =
		BoundedBTreeSet<MogwaiIdOf<T>, ConstU32<MAX_MOGWAIS_PER_PLAYER>>;
	pub(crate) type MogwaiCount = u64;

	pub(crate) const MAX_MOGWAIS_PER_PLAYER: u32 = 24;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Something that provides randomness in the runtime.
		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

		/// The weight information of this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::storage_version(migration::STORAGE_VERSION)]
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

		/// The mogwai is already owned by the sender.
		MogwaiAlreadyOwned,

		/// The mogwai hasn't the necessary rarity.
		MogwaiBadRarity,

		/// Same mogwai chosen for extrinsic.
		MogwaiSame,

		/// Can't hatch mogwai.
		MogwaiNoHatch,

		/// Can't perform specified action while mogwai is on sale.
		MogwaiIsOnSale,

		/// The specified mogwai sells for more than what the sender wants to pay.
		MogwaiNotAffordable,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set organizer, this is a sudo call.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_organizer())]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			Organizer::<T>::put(&organizer);

			// Emit an event.
			Self::deposit_event(Event::OrganizerSet(organizer));

			Ok(())
		}

		/// Update configuration of this sender
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::update_config())]
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

			let mut game_config = GameConfig::new();

			if let Some(config) = AccountConfig::<T>::get(&sender) {
				game_config.parameters = config;
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

			let organizer = Self::organizer().ok_or(Error::<T>::NoOrganizer)?;
			Self::pay_founder(sender.clone(), organizer, price.saturated_into())?;

			game_config.parameters[usize::from(index)] = update_value;

			// updating to the new configuration
			AccountConfig::<T>::insert(&sender, game_config.parameters);

			// Emit an event.
			Self::deposit_event(Event::AccountConfigChanged(sender, game_config.parameters));

			Ok(())
		}

		/// Set price of mogwai.
		#[pallet::weight(T::WeightInfo::set_price())]
		#[pallet::call_index(2)]
		pub fn set_price(
			origin: OriginFor<T>,
			mogwai_id: MogwaiIdOf<T>,
			new_price: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mogwai: MogwaiOf<T> =
				Self::mogwai(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(mogwai.owner == sender, Error::<T>::MogwaiNotOwned);

			MogwaiPrices::<T>::insert(mogwai_id, new_price);
			Self::deposit_event(Event::ForSale(sender, mogwai_id, new_price));

			Ok(())
		}

		/// Clear previously set mogwai price.
		#[pallet::weight(T::WeightInfo::remove_price())]
		#[pallet::call_index(3)]
		pub fn remove_price(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mogwai: MogwaiOf<T> =
				Self::mogwai(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(mogwai.owner == sender, Error::<T>::MogwaiNotOwned);
			ensure!(MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiNotForSale);

			MogwaiPrices::<T>::remove(mogwai_id);
			Self::deposit_event(Event::RemovedFromSale(sender, mogwai_id));

			Ok(())
		}

		/// Create a new mogwai egg.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::create_mogwai())]
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
				id: random_hash_1,
				dna: final_dna,
				genesis: block_number,
				intrinsic: Zero::zero(),
				generation: next_gen,
				rarity: RarityType::from(((max_rarity as u8) << 4) + rarity as u8),
				phase: PhaseType::Bred,
				owner: sender.clone(),
			};

			Self::mint(&sender, random_hash_1, new_mogwai)?;

			// Emit an event.
			Self::deposit_event(Event::MogwaiCreated(sender, random_hash_1));

			Ok(())
		}

		/// Remove a given mogwai.
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove_mogwai())]
		pub fn remove_mogwai(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::organizer().unwrap(), Error::<T>::FounderAction);

			Self::remove(sender.clone(), mogwai_id)?;

			// Emit an event.
			Self::deposit_event(Event::MogwaiRemoved(sender, mogwai_id));

			Ok(())
		}

		/// Transfer mogwai to another account. Mogwais on sale will be unlisted after transfer.
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			mogwai_id: MogwaiIdOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::organizer().unwrap(), Error::<T>::FounderAction);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(to.clone()), Error::<T>::MaxMogwaisInAccount);

			Self::transfer_unchecked(sender.clone(), to.clone(), mogwai_id)?;

			if MogwaiPrices::<T>::contains_key(mogwai_id) {
				MogwaiPrices::<T>::remove(mogwai_id);
			}

			// Emit an event.
			Self::deposit_event(Event::MogwaiTransfered(sender, to, mogwai_id));

			Ok(())
		}

		/// Hatch a mogwai egg
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::hatch_mogwai())]
		pub fn hatch_mogwai(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mut mogwai: MogwaiOf<T> =
				Self::mogwai(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(mogwai.owner == sender, Error::<T>::MogwaiNotOwned);

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
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::sacrifice())]
		pub fn sacrifice(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// TODO this needs to be check, reworked and corrected, add dynasty feature !!!
			let mogwai: MogwaiOf<T> =
				Self::mogwai(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(mogwai.owner == sender, Error::<T>::MogwaiNotOwned);
			ensure!(!MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiIsOnSale);
			ensure!(mogwai.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);

			let intrinsic_to_deposit = {
				let computed_intrinsic =
					mogwai.intrinsic / Pricing::intrinsic_return(mogwai.phase).saturated_into();

				let max_intrinsic =
					BalanceOf::<T>::max_value() - T::Currency::free_balance(&sender);

				sp_std::cmp::min(computed_intrinsic, max_intrinsic)
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
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::sacrifice_into())]
		pub fn sacrifice_into(
			origin: OriginFor<T>,
			mogwai_id_1: T::Hash,
			mogwai_id_2: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Sacrificing into the same mogwai isn't allowed
			ensure!(mogwai_id_1 != mogwai_id_2, Error::<T>::MogwaiSame);

			// TODO this needs to be check, reworked and corrected, add dynasty feature !!!
			let mogwai_1: MogwaiOf<T> =
				Self::mogwai(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			let mut mogwai_2: MogwaiOf<T> =
				Self::mogwai(mogwai_id_2).ok_or(Error::<T>::MogwaiDoesntExists)?;

			ensure!(
				(mogwai_1.owner == sender) && (mogwai_2.owner == sender),
				Error::<T>::MogwaiNotOwned
			);

			ensure!(mogwai_1.rarity != RarityType::Common, Error::<T>::MogwaiBadRarity);
			ensure!(mogwai_2.rarity != RarityType::Common, Error::<T>::MogwaiBadRarity);

			ensure!(mogwai_1.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);
			ensure!(mogwai_2.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);

			ensure!(!MogwaiPrices::<T>::contains_key(mogwai_id_1), Error::<T>::MogwaiIsOnSale);
			ensure!(!MogwaiPrices::<T>::contains_key(mogwai_id_2), Error::<T>::MogwaiIsOnSale);

			let gen_jump = Breeding::sacrifice(
				mogwai_1.generation,
				mogwai_1.rarity,
				&mogwai_1.dna,
				mogwai_2.generation,
				mogwai_2.rarity,
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
		#[pallet::weight(T::WeightInfo::buy_mogwai())]
		#[pallet::call_index(10)]
		pub fn buy_mogwai(
			origin: OriginFor<T>,
			mogwai_id: MogwaiIdOf<T>,
			max_price: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiNotForSale);

			let mogwai: MogwaiOf<T> =
				Self::mogwai(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(mogwai.owner != sender, Error::<T>::MogwaiAlreadyOwned);

			let mogwai_price = Self::mogwai_prices(mogwai_id).unwrap();
			ensure!(mogwai_price <= max_price, Error::<T>::MogwaiNotAffordable);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			T::Currency::transfer(
				&sender,
				&mogwai.owner,
				mogwai_price,
				ExistenceRequirement::KeepAlive,
			)?;

			Self::transfer_unchecked(mogwai.owner.clone(), sender.clone(), mogwai_id)?;

			if MogwaiPrices::<T>::contains_key(mogwai_id) {
				MogwaiPrices::<T>::remove(mogwai_id);
			}

			// TODO: Do something with the results
			let _ = Self::update_achievement_for(&sender, AccountAchievement::Buyer, 1);
			let _ = Self::update_achievement_for(&mogwai.owner, AccountAchievement::Seller, 1);

			// Emit an event.
			Self::deposit_event(Event::MogwaiBought(sender, mogwai.owner, mogwai_id, mogwai_price));

			Ok(())
		}

		/// Morph a mogwai
		#[pallet::weight(T::WeightInfo::morph_mogwai())]
		#[pallet::call_index(11)]
		pub fn morph_mogwai(origin: OriginFor<T>, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// check that the mogwai has an owner and that it is the one calling this extrinsic
			let mut mogwai: MogwaiOf<T> =
				Self::mogwai(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(mogwai.owner == sender, "You don't own this mogwai");
			ensure!(mogwai.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);

			ensure!(!MogwaiPrices::<T>::contains_key(mogwai_id), Error::<T>::MogwaiIsOnSale);

			let pairing_price: BalanceOf<T> =
				Pricing::pairing(mogwai.rarity, mogwai.rarity).saturated_into();

			Self::tip_mogwai(&sender, pairing_price, mogwai_id, &mut mogwai)?;

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
		#[pallet::weight(T::WeightInfo::breed_mogwai())]
		#[pallet::call_index(12)]
		pub fn breed_mogwai(
			origin: OriginFor<T>,
			mogwai_id_1: T::Hash,
			mogwai_id_2: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let mogwai_1: MogwaiOf<T> =
				Self::mogwai(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			let mut mogwai_2: MogwaiOf<T> =
				Self::mogwai(mogwai_id_2).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(mogwai_1.owner == sender, Error::<T>::MogwaiNotOwned);

			// breeding into the same mogwai isn't allowed
			ensure!(mogwai_id_1 != mogwai_id_2, Error::<T>::MogwaiSame);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			ensure!(mogwai_1.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);
			ensure!(mogwai_2.phase != PhaseType::Bred, Error::<T>::MogwaiNoHatch);

			let parents = [mogwai_1.clone(), mogwai_2.clone()];

			let mogwai_id = Self::generate_random_hash(b"breed_mogwai", sender.clone());

			let (rarity, next_gen, max_rarity) = Generation::next_gen(
				parents[0].generation,
				mogwai_1.rarity,
				parents[1].generation,
				mogwai_2.rarity,
				mogwai_id.as_ref(),
			);

			let block_number = <frame_system::Pallet<T>>::block_number();
			let breed_type: BreedType = Self::calculate_breedtype(block_number);

			// add pairing price to mogwai intrinsic value TODO
			let pairing_price: BalanceOf<T> =
				Pricing::pairing(mogwai_1.rarity, mogwai_2.rarity).saturated_into();
			Self::tip_mogwai(&sender, pairing_price, mogwai_id_2, &mut mogwai_2)?;

			let final_dna = Breeding::pairing(breed_type, &mogwai_1.dna[0], &mogwai_2.dna[0]);
			let mogwai_rarity = RarityType::from(((max_rarity as u8) << 4) + rarity as u8);

			let new_mogwai = MogwaiStruct {
				id: mogwai_id,
				dna: final_dna,
				genesis: block_number,
				intrinsic: Zero::zero(),
				generation: next_gen,
				rarity: mogwai_rarity,
				phase: PhaseType::Bred,
				owner: sender.clone(),
			};

			// mint mogwai
			Self::mint(&sender, mogwai_id, new_mogwai)?;

			if mogwai_rarity == RarityType::Mythical {
				// TODO: Do something with the results
				let _ = Self::update_achievement_for(&sender, AccountAchievement::LegendBreeder, 1);
			}

			if mogwai_1.owner != mogwai_2.owner {
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
	fn encode_and_update_nonce() -> Vec<u8> {
		Nonce::<T>::mutate(|nonce| {
			*nonce = nonce.wrapping_add(1);
			*nonce
		})
		.encode()
	}

	fn generate_random_hash(phrase: &[u8], sender: T::AccountId) -> T::Hash {
		let (seed, _) = T::Randomness::random(phrase);
		let decoded_seed =
			<[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref())).unwrap_or_default();
		(decoded_seed, &sender, Self::encode_and_update_nonce()).using_encoded(T::Hashing::hash)
	}

	/// pay fee
	fn pay_fee(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
		let _ = T::Currency::withdraw(
			who,
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
		T::Currency::transfer(&who, &organizer, amount, ExistenceRequirement::KeepAlive)?;

		Ok(())
	}

	/// tiping mogwai
	fn tip_mogwai(
		who: &T::AccountId,
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
		GameConfig::config_value(
			index,
			AccountConfig::<T>::get(&who)
				.map(|config| config[index as usize])
				.unwrap_or_default(),
		)
	}

	///
	fn ensure_not_max_mogwais(who: T::AccountId) -> bool {
		Self::owned_mogwais_count(&who) < Self::config_value(who, 1) as u64
	}

	/// Add mogwai to storage
	fn mint(
		to: &T::AccountId,
		mogwai_id: MogwaiIdOf<T>,
		new_mogwai: MogwaiOf<T>,
	) -> DispatchResult {
		ensure!(!Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiAlreadyExists);

		Mogwais::<T>::insert(mogwai_id, new_mogwai);
		Owners::<T>::try_mutate(to, |id_set| id_set.try_insert(mogwai_id))
			.map_err(|_| Error::<T>::MaxMogwaisInAccount)?;

		AllMogwaisCount::<T>::mutate(|count| {
			*count = count.saturating_add(1);
		});
		OwnedMogwaisCount::<T>::mutate(to, |count| {
			*count = count.saturating_add(1);
		});

		Ok(())
	}

	///
	fn remove(from: T::AccountId, mogwai_id: MogwaiIdOf<T>) -> DispatchResult {
		ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

		Mogwais::<T>::remove(mogwai_id);

		Owners::<T>::mutate(&from, |id_set| {
			id_set.remove(&mogwai_id);
		});

		if MogwaiPrices::<T>::contains_key(mogwai_id) {
			MogwaiPrices::<T>::remove(mogwai_id);
		}

		AllMogwaisCount::<T>::mutate(|count| {
			*count = count.saturating_sub(1);
		});
		OwnedMogwaisCount::<T>::mutate(&from, |count| {
			*count = count.saturating_sub(1);
		});

		Ok(())
	}

	/// transfer mogwai between 'from' account to 'to' account, this method doesn't check for
	/// the validity of the transfer so make sure both accounts can perform the transfer.
	fn transfer_unchecked(
		from: T::AccountId,
		to: T::AccountId,
		mogwai_id: MogwaiIdOf<T>,
	) -> DispatchResult {
		// Update the OwnedMogwaisCount for `from` and `to`
		OwnedMogwaisCount::<T>::mutate(&from, |count| {
			*count = count.saturating_sub(1);
		});
		OwnedMogwaisCount::<T>::mutate(&to, |count| {
			*count = count.saturating_add(1);
		});

		Owners::<T>::mutate(from, |id_set| {
			id_set.remove(&mogwai_id);
		});

		Owners::<T>::try_mutate(&to, |id_set| id_set.try_insert(mogwai_id))
			.map_err(|_| Error::<T>::MaxMogwaisInAccount)?;

		Mogwais::<T>::try_mutate(mogwai_id, |maybe_mogwai| {
			if let Some(mogwai) = maybe_mogwai {
				mogwai.owner = to;
				Ok(())
			} else {
				Err(Error::<T>::MogwaiDoesntExists)
			}
		})?;

		Ok(())
	}

	/// Calculate breed type
	fn calculate_breedtype(block_number: BlockNumberFor<T>) -> BreedType {
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
	fn segment_and_bake(mogwai: MogwaiOf<T>, hash: T::Hash) -> ([[u8; 32]; 2], RarityType) {
		let block_hash = unsafe {
			let mut block_hash: MaybeUninit<[u8; 32]> = MaybeUninit::uninit();
			let block_hash_ptr = block_hash.as_mut_ptr() as *mut u8;
			copy_nonoverlapping(hash.as_ref()[0..32].as_ptr(), block_hash_ptr, 32);
			block_hash.assume_init()
		};

		// segment and and bake the hatched mogwai
		(Breeding::segmenting(mogwai.dna, block_hash), Breeding::bake(mogwai.rarity, block_hash))
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
