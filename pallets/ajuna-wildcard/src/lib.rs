#![allow(dead_code)]
#![cfg_attr(not(feature = "std"), no_std)]

mod asset;
#[cfg(test)]
mod tests;

pub use pallet::*;

pub use asset::{OnMappingRequest, WideId};

use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::{
			fungibles, nonfungibles_v2, ExistenceRequirement::AllowDeath, Fortitude::Polite,
			Precision::Exact, Preservation::Expendable,
		},
		Currency, ReservableCurrency, Time,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_core::sp_std;
use sp_runtime::traits::{AccountIdConversion, SaturatedConversion, Verify};

use asset::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::tokens::{
		fungibles::{
			Create as CreateFungibles, Inspect as InspectFungibles, Mutate as MutateFungibles,
		},
		nonfungibles_v2::{
			Create as CreateNonFungibles, Inspect as InspectNonFungibles,
			Mutate as MutateNonFungibles, Transfer,
		},
	};

	pub(crate) type MomentOf<T> = <<T as Config>::Time as Time>::Moment;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type AssetBalanceOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;
	pub(crate) type DepositValueKindOf<T> = DepositValueKind<BalanceOf<T>, AssetBalanceOf<T>>;
	pub(crate) type MappingKey = (AssetOrigin, AssetType, WideId);

	pub(crate) type AssetIdOf<T> = <T as Config>::AssetId;
	pub(crate) type CollectionIdOf<T> = <T as Config>::CollectionId;
	pub(crate) type ItemIdOf<T> = <T as Config>::ItemId;
	pub(crate) type AssetOf<T> = Asset<AssetIdOf<T>, CollectionIdOf<T>, ItemIdOf<T>>;
	pub(crate) type NftAddressOf<T> =
		NftAddress<<T as Config>::CollectionId, <T as Config>::ItemId>;

	#[pallet::storage]
	pub type Administrator<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type StartTime<T: Config> = StorageValue<_, MomentOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type EpochDuration<T: Config> = StorageValue<_, MomentOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type SignerKey<T: Config> = StorageValue<_, sp_core::sr25519::Public, OptionQuery>;

	#[pallet::storage]
	pub type Frozen<T: Config> = StorageValue<_, EpochNumber, OptionQuery>;

	#[pallet::storage]
	pub type Deposits<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, EpochNumber>,
			NMapKey<Identity, T::AccountId>,
			NMapKey<Blake2_128Concat, AssetOf<T>>,
		),
		DepositValueKindOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub type Withdrawals<T: Config> = StorageDoubleMap<
		_,
		Identity,
		T::AccountId,
		Blake2_128Concat,
		AssetOf<T>,
		(EpochNumber, ChunkIndex),
		OptionQuery,
	>;

	#[pallet::storage]
	pub type Challenges<T: Config> =
		StorageMap<_, Identity, (EpochNumber, T::AccountId), ChunkIndex, OptionQuery>;

	#[pallet::storage]
	pub type AssetIdMapping<T: Config> =
		StorageMap<_, Blake2_128, MappingKey, AssetIdOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type CollectionIdMapping<T: Config> =
		StorageMap<_, Blake2_128, MappingKey, CollectionIdOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type ItemIdMapping<T: Config> =
		StorageMap<_, Blake2_128, MappingKey, ItemIdOf<T>, OptionQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// Pallet administrator account not set, the account needs to be present before adjusting
		/// parameters.
		AdministratorNotSet,
		/// Tried to change pallet parameters using a non-admin account.
		AccountIsNotAdministrator,
		/// Failed to decode account identifier from the BalanceProof/ZeroProof input.
		FailedToDecodeAccount,
		/// The fungible asset's quantity from the AssetDeposit input cannot fit in a u128.
		FungibleAssetQuantityOverflow,
		/// The mapping for a native asset's identifiers cannot be located during asset withdrawal.
		NativeAssetMappingNotFound,
		/// The mapping for a foreign asset's identifiers cannot be located during asset deposit.
		NonNativeAssetMappingNotFound,
		/// The asset type found in the BalanceProof/ZeroProof input could not be mapped to any
		/// know type.
		UnknownAssetTypeInProof,
		/// Tried to mutate a fungible asset's deposit using a non fungible asset's data.
		InvalidFungibleDepositMutation,
		/// Tried to reserve a fungible asset using a non fungible asset's data.
		ReservedFungibleWithNonFungibleValue,
		/// Tried to undo a reserve of a fungible asset using a non fungible asset's data.
		UnreservedFungibleWithNonFungibleValue,
		/// Tried to a reserve a non fungible asset that has no owner data.
		ReserveNonFungibleWithoutOwner,
		/// The padding for the native fungible asset doesn't match the expect padding value.
		InvalidNativeFungiblePadding,
		/// The padding for the native non-fungible asset doesn't match the expect padding value.
		InvalidNativeNonFungiblePadding,
		/// The amount of reserved fungible asset funds is lower than the requested withdrawal.
		InsufficientReserveFunds,
		/// The balance proof origin identifier doesn't match the expected chain identifier.
		BalanceProofWrongOriginNetwork,
		/// The exit flag is not properly set.
		BadExitFlag,
		/// The public signer key cannot be found, set it before trying to validate signatures.
		MissingPublicKey,
		/// The signature doesn't match the expected value.
		BadSignature,
		/// The pallet's StartTime or EpochDuration parameters have not been set.
		TimeNotSet,
		/// Withdrawal is not in the proper sequence with previous withdrawals.
		WithdrawNotInline,
		/// The balance proof is still not finalize, so it cannot be used until later.
		NonFinalizedBalanceProof,
		/// The owner of the non-fungible asset is not the same that wants to reserve it.
		ItemNotOwned,
		/// This epoch has already been challenged previously.
		AlreadyChallengedEpoch,
		/// The epoch number is not correct.
		InvalidEpochNumber,
		/// The challenger account doesn't have enough free balance to perform the challenging.
		InsufficientChallengeBalance,
		/// The balance proof chunk index doesn't match the expected challenge index.
		WrongChunkRespondedChallenge,
		/// The balance proof key doesn't match any active challenge.
		WronglyRespondedChallenge,
		/// There are not active challenges, so there's no reason to freeze the pallet.
		NothingToFreeze,
		/// The pallet is not frozen for withdrawals.
		NotFrozen,
		/// The pallet is frozen, so deposits and withdrawals are blocked.
		PalletFrozen,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AdministratorSet {
			administrator: T::AccountId,
		},
		ParametersSet {
			start_time: Option<MomentOf<T>>,
			epoch_duration: Option<MomentOf<T>>,
			signer_key: Option<sp_core::sr25519::Public>,
		},
		AssetDeposit {
			epoch: EpochNumber,
			depositor: T::AccountId,
			asset_origin: AssetOrigin,
			asset_type: AssetType,
			primary_id: WideId,
			secondary_id: WideId,
		},
		AssetWithdraw {
			epoch: EpochNumber,
			withdrawer: T::AccountId,
			asset_origin: AssetOrigin,
			asset_type: AssetType,
			primary_id: WideId,
			secondary_id: WideId,
		},
		FrozenAssetWithdraw {
			epoch: EpochNumber,
			withdrawer: T::AccountId,
			asset_origin: AssetOrigin,
			asset_type: AssetType,
			primary_id: WideId,
			secondary_id: WideId,
		},
		DepositsRefunded {
			epoch: EpochNumber,
			beneficieary: T::AccountId,
		},
		ChallengeResponded {
			challenger: T::AccountId,
			challenged_epoch: EpochNumber,
			balance_proof: BalanceProof,
		},
		ChallengeZeroResponded {
			challenger: T::AccountId,
			challenged_epoch: EpochNumber,
			balance_proof: ZeroBalanceProof,
		},
		ChallengeCalled {
			epoch: EpochNumber,
			challenger: T::AccountId,
		},
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The pallet's id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: ReservableCurrency<Self::AccountId>;

		type AssetId: Member + Parameter + Clone + MaybeSerializeDeserialize + MaxEncodedLen;

		type Fungibles: fungibles::Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = BalanceOf<Self>>
			+ fungibles::Create<Self::AccountId>
			+ fungibles::Destroy<Self::AccountId>
			+ fungibles::Mutate<Self::AccountId>;

		/// Type that holds the specific configurations for a collection of
		/// non-fungibles.
		type CollectionConfig: Default + MaxEncodedLen + TypeInfo;

		/// Type that holds the specific configurations for a non-fungible item.
		type ItemConfig: Default + MaxEncodedLen + TypeInfo;

		type NonFungibles: nonfungibles_v2::Inspect<
				Self::AccountId,
				CollectionId = Self::CollectionId,
				ItemId = Self::ItemId,
			> + nonfungibles_v2::Create<Self::AccountId, Self::CollectionConfig>
			+ nonfungibles_v2::Destroy<Self::AccountId>
			+ nonfungibles_v2::Mutate<Self::AccountId, Self::ItemConfig>
			+ nonfungibles_v2::Transfer<Self::AccountId>;

		type CollectionId: Member + Parameter + MaxEncodedLen + Copy;
		type ItemId: Member + Parameter + MaxEncodedLen + Copy;

		type OnMappingRequest: OnMappingRequest<Self::AssetId, Self::CollectionId, Self::ItemId>;

		type Time: Time;

		type ChainId: Get<AssetOrigin>;
		type NativeTokenAssetId: Get<AssetIdOf<Self>>;

		/// Minimum amount of free balance in an account wishing to challenge an epoch
		type ChallengeMinBalance: Get<BalanceOf<Self>>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set a administrator account.
		///
		/// This call allows setting an account to act as an administrator. It must be called with
		/// root privilege.
		#[pallet::weight({10_000})]
		#[pallet::call_index(0)]
		pub fn set_administrator(
			origin: OriginFor<T>,
			administrator: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;
			Administrator::<T>::put(&administrator);

			Self::deposit_event(Event::AdministratorSet { administrator });
			Ok(())
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(1)]
		pub fn set_parameters(
			origin: OriginFor<T>,
			start_time: Option<MomentOf<T>>,
			epoch_duration: Option<MomentOf<T>>,
			signer_key: Option<sp_core::sr25519::Public>,
		) -> DispatchResult {
			let admin = Administrator::<T>::get();
			ensure!(admin.is_some(), Error::<T>::AdministratorNotSet);

			let account = ensure_signed(origin)?;
			ensure!(admin.unwrap() == account, Error::<T>::AccountIsNotAdministrator);

			if let Some(start_time) = start_time {
				StartTime::<T>::put(start_time);
			}

			if let Some(epoch_duration) = epoch_duration {
				EpochDuration::<T>::put(epoch_duration);
			}

			if let Some(signer_key) = signer_key {
				SignerKey::<T>::put(signer_key);
			}

			Self::deposit_event(Event::ParametersSet {
				start_time: StartTime::<T>::get(),
				epoch_duration: EpochDuration::<T>::get(),
				signer_key: SignerKey::<T>::get(),
			});
			Ok(())
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(2)]
		pub fn deposit(origin: OriginFor<T>, asset_deposit: AssetDeposit) -> DispatchResult {
			ensure!(!Frozen::<T>::exists(), Error::<T>::PalletFrozen);

			let depositor = ensure_signed(origin)?;
			let (asset, value) = Self::validate_and_convert_deposit(asset_deposit)?;

			let deposit_epoch = match &asset.kind {
				AssetKind::Fungible(asset_id) =>
					Self::reserve_fungibles(&depositor, &asset.origin, asset_id, &value)
						.and_then(|_| Self::insert_deposit(&asset, &depositor, &value)),
				AssetKind::NonFungible(addr) =>
					Self::reserve_non_fungibles(&depositor, &asset.origin, addr).and_then(|_| {
						Self::insert_deposit(&asset, &depositor, &DepositValueKind::NonFungible)
					}),
			}?;

			Self::deposit_event(Event::<T>::AssetDeposit {
				epoch: deposit_epoch,
				depositor,
				asset_origin: asset_deposit.origin,
				asset_type: asset_deposit.asset_type,
				primary_id: asset_deposit.primary_id,
				secondary_id: asset_deposit.secondary_id,
			});

			if deposit_epoch > 4 {
				Self::clear_old_deposit_entries(deposit_epoch.saturating_sub(4))?;
			}

			Ok(())
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(3)]
		pub fn withdraw(
			origin: OriginFor<T>,
			balance_proof: BalanceProof,
			signature: sp_core::sr25519::Signature,
		) -> DispatchResult {
			let withdrawer = ensure_signed(origin)?;
			let (asset, value, epoch_num) =
				Self::validate_and_convert_proof(&withdrawer, &balance_proof, &signature)?;

			match &asset.kind {
				AssetKind::Fungible(asset_id) =>
					Self::unreserve_fungibles(&withdrawer, &asset.origin, asset_id, &value),
				AssetKind::NonFungible(addr) =>
					Self::unreserve_non_fungibles(&withdrawer, &asset.origin, addr),
			}?;

			Self::register_withdrawal(epoch_num, &withdrawer, &asset)?;

			Self::deposit_event(Event::<T>::AssetWithdraw {
				epoch: balance_proof.epoch,
				withdrawer,
				asset_origin: balance_proof.origin,
				asset_type: AssetType::from(balance_proof.asset_type),
				primary_id: balance_proof.primary_id,
				secondary_id: balance_proof.secondary_id,
			});

			Ok(())
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(4)]
		pub fn withdraw_frozen(
			origin: OriginFor<T>,
			balance_proof: BalanceProof,
			signature: sp_core::sr25519::Signature,
		) -> DispatchResult {
			let withdrawer = ensure_signed(origin)?;

			let maybe_frozen_epoch = Frozen::<T>::get();
			ensure!(maybe_frozen_epoch.is_some(), Error::<T>::NotFrozen);
			ensure!(
				maybe_frozen_epoch.unwrap() == balance_proof.epoch,
				Error::<T>::InvalidEpochNumber
			);

			let (asset, value, epoch_num) =
				Self::validate_and_convert_proof(&withdrawer, &balance_proof, &signature)?;

			match &asset.kind {
				AssetKind::Fungible(asset_id) =>
					Self::unreserve_fungibles(&withdrawer, &asset.origin, asset_id, &value),
				AssetKind::NonFungible(addr) =>
					Self::unreserve_non_fungibles(&withdrawer, &asset.origin, addr),
			}?;

			Self::register_withdrawal(epoch_num, &withdrawer, &asset)?;

			Self::deposit_event(Event::<T>::FrozenAssetWithdraw {
				epoch: balance_proof.epoch,
				withdrawer,
				asset_origin: balance_proof.origin,
				asset_type: AssetType::from(balance_proof.asset_type),
				primary_id: balance_proof.primary_id,
				secondary_id: balance_proof.secondary_id,
			});

			Ok(())
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(5)]
		pub fn refund_frozen(origin: OriginFor<T>) -> DispatchResult {
			let withdrawer = ensure_signed(origin)?;

			// POSSIBLE IMPROVEMENT
			// can also be done for an other account, like claim can be done for claimFor().

			// one for the own account
			// one for any other account
			// one for all accounts

			let frozen_epoch = {
				let frozen = Frozen::<T>::get();
				ensure!(frozen.is_some(), Error::<T>::NotFrozen);
				frozen.unwrap()
			};

			let deposits_to_refund = Deposits::<T>::iter_keys()
				.filter(|(epoch, account, _)| epoch > &frozen_epoch && account == &withdrawer)
				.collect::<sp_std::vec::Vec<_>>();

			for (epoch, account, asset) in deposits_to_refund {
				if let Some(deposit_value) = Deposits::<T>::take((&epoch, &account, &asset)) {
					match &asset.kind {
						AssetKind::Fungible(asset_id) => Self::unreserve_fungibles(
							&withdrawer,
							&asset.origin,
							asset_id,
							&deposit_value,
						),
						AssetKind::NonFungible(addr) =>
							Self::unreserve_non_fungibles(&withdrawer, &asset.origin, addr),
					}?;
				}
			}

			Self::deposit_event(Event::<T>::DepositsRefunded {
				epoch: frozen_epoch,
				beneficieary: withdrawer,
			});

			Ok(())
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(6)]
		pub fn challenge(origin: OriginFor<T>) -> DispatchResult {
			let challenger = ensure_signed(origin)?;

			ensure!(
				T::Currency::free_balance(&challenger) >= T::ChallengeMinBalance::get(),
				Error::<T>::InsufficientChallengeBalance
			);

			let epoch_number = Self::calculate_epoch_number_from(T::Time::now())?;
			ensure!(epoch_number > 0, Error::<T>::InvalidEpochNumber);

			ensure!(
				!Challenges::<T>::contains_key((epoch_number - 1, challenger.clone())),
				Error::<T>::AlreadyChallengedEpoch
			);

			Challenges::<T>::insert((epoch_number - 1, challenger.clone()), 0 as ChunkIndex);

			Self::deposit_event(Event::<T>::ChallengeCalled {
				epoch: epoch_number - 1,
				challenger,
			});

			Ok(())
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(7)]
		pub fn respond_challenge(
			origin: OriginFor<T>,
			balance_proof: BalanceProof,
			signature: sp_core::sr25519::Signature,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let challenger = T::AccountId::decode(&mut &balance_proof.account[..])
				.map_err(|_| Error::<T>::FailedToDecodeAccount)?;

			let challenge_key = (balance_proof.epoch, challenger.clone());

			if let Some(chunk_index) = Challenges::<T>::get(&challenge_key) {
				if chunk_index == balance_proof.chunk_index {
					Self::validate_proof_signature(&balance_proof, &signature)?;

					Self::deposit_event(Event::<T>::ChallengeResponded {
						challenger,
						challenged_epoch: balance_proof.epoch,
						balance_proof,
					});

					if chunk_index == balance_proof.chunk_last {
						Challenges::<T>::remove(&challenge_key);
					} else {
						Challenges::<T>::insert(challenge_key, chunk_index.saturating_add(1));
					}
					Ok(())
				} else {
					Err(Error::<T>::WrongChunkRespondedChallenge.into())
				}
			} else {
				Err(Error::<T>::WronglyRespondedChallenge.into())
			}
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(8)]
		pub fn freeze(origin: OriginFor<T>) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(!Frozen::<T>::exists(), Error::<T>::PalletFrozen);

			let epoch_number = Self::calculate_epoch_number_from(T::Time::now())?;
			ensure!(epoch_number >= 4, Error::<T>::InvalidEpochNumber);

			if Challenges::<T>::iter_keys().any(|(epoch, _)| epoch == (epoch_number - 3)) {
				Frozen::<T>::set(Some(epoch_number - 4));
				Ok(())
			} else {
				Err(Error::<T>::NothingToFreeze.into())
			}
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(9)]
		pub fn propagate_freeze(
			origin: OriginFor<T>,
			freeze_proof: FreezeProof,
			client_key: sp_core::sr25519::Public,
			client_key_signature: sp_core::sr25519::Signature,
			proof_signature: sp_core::sr25519::Signature,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let signer = SignerKey::<T>::get()
				.ok_or::<DispatchError>(Error::<T>::MissingPublicKey.into())?;

			let client_key_msg = {
				let mut bytes = sp_std::vec::Vec::with_capacity(0);

				bytes.extend(LIGHT_CLIENT_PROOF_PREFIX.to_vec());
				bytes.extend(client_key.to_vec());

				bytes
			};

			ensure!(
				client_key_signature.verify(client_key_msg.as_slice(), &signer),
				Error::<T>::BadSignature
			);

			let msg = freeze_proof.extract_msg();
			ensure!(proof_signature.verify(msg.as_slice(), &client_key), Error::<T>::BadSignature);

			let epoch_number = Self::calculate_epoch_number_from(T::Time::now())?;
			ensure!(epoch_number >= 4, Error::<T>::InvalidEpochNumber);
			ensure!(epoch_number - 4 == freeze_proof.epoch, Error::<T>::InvalidEpochNumber);
			Frozen::<T>::set(Some(freeze_proof.epoch));

			Ok(())
		}

		#[pallet::weight({10_000})]
		#[pallet::call_index(10)]
		pub fn respond_zero_challenge(
			origin: OriginFor<T>,
			zero_proof: ZeroBalanceProof,
			signature: sp_core::sr25519::Signature,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let challenger = T::AccountId::decode(&mut &zero_proof.account[..])
				.map_err(|_| Error::<T>::FailedToDecodeAccount)?;

			let challenge_key = (zero_proof.epoch, challenger.clone());

			if Challenges::<T>::get(&challenge_key).is_some() {
				Self::validate_proof_signature(&zero_proof, &signature)?;

				Self::deposit_event(Event::<T>::ChallengeZeroResponded {
					challenger,
					challenged_epoch: zero_proof.epoch,
					balance_proof: zero_proof,
				});

				Challenges::<T>::remove(&challenge_key);

				Ok(())
			} else {
				Err(Error::<T>::WronglyRespondedChallenge.into())
			}
		}
	}

	impl<T: Config> Pallet<T> {
		#[inline]
		/// The account identifier of the pallet's account.
		pub(crate) fn reserve_account() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		#[inline]
		fn is_native(asset_origin: &AssetOrigin) -> bool {
			*asset_origin == T::ChainId::get()
		}

		#[inline]
		fn is_native_chain_token(asset_id: &AssetIdOf<T>) -> bool {
			*asset_id == T::NativeTokenAssetId::get()
		}

		pub(crate) fn calculate_epoch_number_from(
			epoch: MomentOf<T>,
		) -> Result<EpochNumber, DispatchError> {
			let start_time = StartTime::<T>::get();
			let epoch_duration = EpochDuration::<T>::get();

			ensure!(start_time.is_some() && epoch_duration.is_some(), Error::<T>::TimeNotSet);

			let epoch_number =
				((epoch - start_time.unwrap()) / epoch_duration.unwrap()).saturated_into::<u64>();

			Ok(epoch_number)
		}

		fn validate_deposit(asset: &AssetDeposit) -> DispatchResult {
			let asset_id_padding = asset.primary_id[0..30].as_ref();

			let cmp_fn = if Self::is_native(&asset.origin) {
				sp_std::cmp::PartialEq::eq
			} else {
				sp_std::cmp::PartialEq::ne
			};

			match asset.asset_type {
				AssetType::Fungible => ensure!(
					cmp_fn(asset_id_padding, NATIVE_FUNGIBLE_PAD.as_slice()),
					Error::<T>::InvalidNativeFungiblePadding
				),
				AssetType::NonFungible => ensure!(
					cmp_fn(asset_id_padding, NATIVE_NON_FUNGIBLE_PAD.as_slice()),
					Error::<T>::InvalidNativeNonFungiblePadding
				),
			}

			Ok(())
		}

		fn validate_and_convert_deposit(
			asset: AssetDeposit,
		) -> Result<(AssetOf<T>, DepositValueKindOf<T>), DispatchError> {
			Self::validate_deposit(&asset)?;

			let is_native = Self::is_native(&asset.origin);

			let mapping_key = (asset.origin, asset.asset_type, asset.primary_id);

			match asset.asset_type {
				AssetType::Fungible => {
					let asset_id = if let Some(asset_id) = AssetIdMapping::<T>::get(mapping_key) {
						asset_id
					} else if is_native {
						let asset_id =
							T::OnMappingRequest::on_fungible_asset_mapping(asset.primary_id);

						AssetIdMapping::<T>::insert(mapping_key, asset_id.clone());

						asset_id
					} else {
						// Mapping should have been defined at withdraw
						return Err(Error::<T>::NonNativeAssetMappingNotFound.into())
					};

					let converted_asset =
						Asset { origin: asset.origin, kind: AssetKind::Fungible(asset_id.clone()) };

					let asset_bytes_1 = <&[u8; 16]>::try_from(&asset.secondary_id[0..16]).unwrap();
					let asset_qty =
						u128::from_le_bytes(*asset_bytes_1).saturated_into::<BalanceOf<T>>();

					let asset_bytes_2 = <&[u8; 16]>::try_from(&asset.secondary_id[16..32]).unwrap();
					let asset_check = u128::from_le_bytes(*asset_bytes_2);

					if asset_check > 0 {
						return Err(Error::<T>::FungibleAssetQuantityOverflow.into())
					}

					let value = if is_native && asset_id.eq(&T::NativeTokenAssetId::get()) {
						DepositValueKind::Fungible(DepositValue::Token(asset_qty))
					} else {
						DepositValueKind::Fungible(DepositValue::Asset(asset_qty))
					};

					Ok((converted_asset, value))
				},
				AssetType::NonFungible => {
					let collection_id = if let Some(collection_id) =
						CollectionIdMapping::<T>::get(mapping_key)
					{
						collection_id
					} else if is_native {
						let collection_id = T::OnMappingRequest::on_non_fungible_collection_mapping(
							asset.primary_id,
						);

						CollectionIdMapping::<T>::insert(mapping_key, collection_id);

						collection_id
					} else {
						// Mapping should have been defined at withdraw
						return Err(Error::<T>::NonNativeAssetMappingNotFound.into())
					};

					let mapping_item_key = (asset.origin, asset.asset_type, asset.secondary_id);

					let item_id = if let Some(item_id) = ItemIdMapping::<T>::get(mapping_item_key) {
						item_id
					} else if is_native {
						let item_id =
							T::OnMappingRequest::on_non_fungible_item_mapping(asset.secondary_id);

						ItemIdMapping::<T>::insert(mapping_item_key, item_id);

						item_id
					} else {
						// Mapping should have been defined at withdraw
						return Err(Error::<T>::NonNativeAssetMappingNotFound.into())
					};

					let converted_asset = Asset {
						origin: asset.origin,
						kind: AssetKind::NonFungible(NftAddress(collection_id, item_id)),
					};

					Ok((converted_asset, DepositValueKind::NonFungible))
				},
			}
		}

		fn validate_proof(
			withdrawer: &T::AccountId,
			asset: &AssetOf<T>,
			proof: &BalanceProof,
			epoch_number: EpochNumber,
			signature: &sp_core::sr25519::Signature,
		) -> DispatchResult {
			ensure!(epoch_number >= proof.epoch + 4, Error::<T>::NonFinalizedBalanceProof);

			if let Some((epoch, chunk_index)) = Withdrawals::<T>::get(withdrawer, asset) {
				ensure!(
					proof.epoch > epoch ||
						(proof.epoch == epoch && chunk_index < proof.chunk_index),
					Error::<T>::WithdrawNotInline
				);
			}

			// For testing we want to also check with foreign
			// proofs to verify all logic paths
			#[cfg(not(test))]
			ensure!(proof.origin == T::ChainId::get(), Error::<T>::BalanceProofWrongOriginNetwork);
			Self::validate_proof_signature(proof, signature)?;
			ensure!(proof.exit_flag, Error::<T>::BadExitFlag);

			Ok(())
		}

		fn validate_proof_signature<P: Proof>(
			proof: &P,
			signature: &sp_core::sr25519::Signature,
		) -> DispatchResult {
			let msg = proof.extract_msg();
			let signer = SignerKey::<T>::get()
				.ok_or::<DispatchError>(Error::<T>::MissingPublicKey.into())?;
			ensure!(signature.verify(msg.as_slice(), &signer), Error::<T>::BadSignature);
			Ok(())
		}

		fn validate_and_convert_proof(
			withdrawer: &T::AccountId,
			proof: &BalanceProof,
			signature: &sp_core::sr25519::Signature,
		) -> Result<(AssetOf<T>, DepositValueKindOf<T>, EpochNumber), DispatchError> {
			let is_native = Self::is_native(&proof.origin);

			let mapping_key = (proof.origin, AssetType::from(proof.asset_type), proof.primary_id);

			let (asset, value) = if proof.asset_type == AssetType::Fungible as u8 {
				let asset_id = if let Some(asset_id) = AssetIdMapping::<T>::get(mapping_key) {
					asset_id
				} else if !is_native {
					let asset_id = T::OnMappingRequest::on_fungible_asset_mapping(proof.primary_id);

					AssetIdMapping::<T>::insert(mapping_key, asset_id.clone());

					asset_id
				} else {
					// Mapping should have been defined at deposit
					return Err(Error::<T>::NativeAssetMappingNotFound.into())
				};

				let converted_asset =
					Asset { origin: proof.origin, kind: AssetKind::Fungible(asset_id.clone()) };

				let asset_bytes_1 = <&[u8; 16]>::try_from(&proof.secondary_id[0..16]).unwrap();
				let asset_qty =
					u128::from_le_bytes(*asset_bytes_1).saturated_into::<BalanceOf<T>>();

				let asset_bytes_2 = <&[u8; 16]>::try_from(&proof.secondary_id[16..32]).unwrap();
				let asset_check = u128::from_le_bytes(*asset_bytes_2);

				if asset_check > 0 {
					return Err(Error::<T>::FungibleAssetQuantityOverflow.into())
				}

				let value = if is_native && asset_id == T::NativeTokenAssetId::get() {
					DepositValueKind::Fungible(DepositValue::Token(asset_qty))
				} else {
					DepositValueKind::Fungible(DepositValue::Asset(asset_qty))
				};

				Ok((converted_asset, value))
			} else if proof.asset_type == AssetType::NonFungible as u8 {
				let collection_id =
					if let Some(collection_id) = CollectionIdMapping::<T>::get(mapping_key) {
						collection_id
					} else if !is_native {
						let reserve_account = Self::reserve_account();

						let collection_id = T::NonFungibles::create_collection(
							&reserve_account,
							&reserve_account,
							&T::CollectionConfig::default(),
						)?;

						CollectionIdMapping::<T>::insert(mapping_key, collection_id);

						collection_id
					} else {
						// Mapping should have been defined at deposit
						return Err(Error::<T>::NativeAssetMappingNotFound.into())
					};

				let mapping_item_key =
					(proof.origin, AssetType::from(proof.asset_type), proof.secondary_id);

				let item_id = if let Some(item_id) = ItemIdMapping::<T>::get(mapping_item_key) {
					item_id
				} else if !is_native {
					let item_id =
						T::OnMappingRequest::on_non_fungible_item_mapping(proof.secondary_id);

					ItemIdMapping::<T>::insert(mapping_item_key, item_id);

					item_id
				} else {
					// Mapping should have been defined at deposit
					return Err(Error::<T>::NativeAssetMappingNotFound.into())
				};

				let converted_asset = Asset {
					origin: proof.origin,
					kind: AssetKind::NonFungible(NftAddress(collection_id, item_id)),
				};

				Ok((converted_asset, DepositValueKind::NonFungible))
			} else {
				Err(Error::<T>::UnknownAssetTypeInProof)
			}?;

			let epoch_number = Self::calculate_epoch_number_from(T::Time::now())?;
			Self::validate_proof(withdrawer, &asset, proof, epoch_number, signature)?;

			Ok((asset, value, proof.epoch))
		}

		fn insert_deposit(
			asset: &AssetOf<T>,
			depositor: &T::AccountId,
			value: &DepositValueKindOf<T>,
		) -> Result<EpochNumber, DispatchError> {
			let epoch = Self::calculate_epoch_number_from(T::Time::now())?;

			match Deposits::<T>::take((epoch, depositor, asset)) {
				Some(mut deposit_value) => {
					if let DepositValueKind::Fungible(DepositValue::Token(ref mut existing_value)) =
						deposit_value
					{
						match value {
							DepositValueKind::Fungible(DepositValue::Token(v)) => {
								*existing_value += *v;
								Deposits::<T>::insert((epoch, depositor, asset), &deposit_value);
								Ok(())
							},
							_ => Err(Error::<T>::InvalidFungibleDepositMutation),
						}?;
					}
					if let DepositValueKind::Fungible(DepositValue::Asset(ref mut existing_value)) =
						deposit_value
					{
						match value {
							DepositValueKind::Fungible(DepositValue::Asset(v)) => {
								*existing_value += *v;
								Deposits::<T>::insert((epoch, depositor, asset), &deposit_value);
								Ok(())
							},
							_ => Err(Error::<T>::InvalidFungibleDepositMutation),
						}?;
					}
				},
				None => Deposits::<T>::insert((epoch, depositor, asset), value),
			}

			Ok(epoch)
		}

		/// Reserve or lock the `value` amount of the given asset, which can either be native or
		/// foreign.
		fn reserve_fungibles(
			who: &T::AccountId,
			asset_origin: &AssetOrigin,
			asset_id: &AssetIdOf<T>,
			value: &DepositValueKindOf<T>,
		) -> DispatchResult {
			if Self::is_native(asset_origin) {
				let reserve_account = Self::reserve_account();
				if Self::is_native_chain_token(asset_id) {
					let amount = match value {
						DepositValueKind::Fungible(DepositValue::Token(value)) => Ok(value),
						_ => Err(Error::<T>::ReservedFungibleWithNonFungibleValue),
					}?;

					T::Currency::transfer(who, &reserve_account, *amount, AllowDeath)
				} else {
					let amount = match value {
						DepositValueKind::Fungible(DepositValue::Asset(value)) => Ok(value),
						_ => Err(Error::<T>::ReservedFungibleWithNonFungibleValue),
					}?;

					T::Fungibles::transfer(
						asset_id.clone(),
						who,
						&reserve_account,
						*amount,
						Expendable,
					)
					.map(|_| ())
				}
			} else {
				let amount = match value {
					DepositValueKind::Fungible(DepositValue::Asset(value)) => Ok(value),
					_ => Err(Error::<T>::ReservedFungibleWithNonFungibleValue),
				}?;
				T::Fungibles::burn_from(asset_id.clone(), who, *amount, Exact, Polite).map(|_| ())
			}
		}

		/// Unreserve or unlock the `value` amount of the given asset, which can either be native or
		/// foreign.
		fn unreserve_fungibles(
			who: &T::AccountId,
			asset_origin: &AssetOrigin,
			asset_id: &AssetIdOf<T>,
			value: &DepositValueKindOf<T>,
		) -> DispatchResult {
			let reserve_account = Self::reserve_account();

			if Self::is_native(asset_origin) {
				if Self::is_native_chain_token(asset_id) {
					let withdrawal_amount = match value {
						DepositValueKind::Fungible(DepositValue::Token(v)) => Ok(v),
						_ => Err(Error::<T>::UnreservedFungibleWithNonFungibleValue),
					}?;

					let available_balance = T::Currency::free_balance(&reserve_account);

					ensure!(
						withdrawal_amount <= &available_balance,
						Error::<T>::InsufficientReserveFunds
					);

					T::Currency::transfer(&reserve_account, who, *withdrawal_amount, AllowDeath)?;
				} else {
					let withdrawal_amount = match value {
						DepositValueKind::Fungible(DepositValue::Asset(v)) => Ok(v),
						_ => Err(Error::<T>::UnreservedFungibleWithNonFungibleValue),
					}?;

					let available_balance =
						T::Fungibles::balance(asset_id.clone(), &reserve_account);

					ensure!(
						withdrawal_amount <= &available_balance,
						Error::<T>::InsufficientReserveFunds
					);

					T::Fungibles::transfer(
						asset_id.clone(),
						&reserve_account,
						who,
						*withdrawal_amount,
						Expendable,
					)?;
				}
				Ok(())
			} else {
				let withdrawal_amount = match value {
					DepositValueKind::Fungible(DepositValue::Asset(v)) => Ok(v),
					_ => Err(Error::<T>::UnreservedFungibleWithNonFungibleValue),
				}?;

				if !T::Fungibles::asset_exists(asset_id.clone()) {
					T::Fungibles::create(
						asset_id.clone(),
						reserve_account,
						false,
						1_u32.saturated_into(),
					)?;
				}

				T::Fungibles::mint_into(asset_id.clone(), who, *withdrawal_amount).map(|_| ())
			}
		}

		// Reserve or lock the given NFT.
		fn reserve_non_fungibles(
			who: &T::AccountId,
			asset_origin: &AssetOrigin,
			addr: &NftAddressOf<T>,
		) -> DispatchResult {
			let (collection_id, item_id) = (&addr.0, &addr.1);

			let item_owner = T::NonFungibles::owner(collection_id, item_id);

			ensure!(item_owner.is_some(), Error::<T>::ReserveNonFungibleWithoutOwner);
			ensure!(item_owner.as_ref() == Some(who), Error::<T>::ItemNotOwned);

			if Self::is_native(asset_origin) {
				T::NonFungibles::transfer(collection_id, item_id, &Self::reserve_account())
			} else {
				T::NonFungibles::burn(collection_id, item_id, Some(who))
			}
		}

		/// Unreserve or unlock the given NFT.
		fn unreserve_non_fungibles(
			who: &T::AccountId,
			asset_origin: &AssetOrigin,
			addr: &NftAddressOf<T>,
		) -> DispatchResult {
			let (collection_id, item_id) = (&addr.0, &addr.1);

			if Self::is_native(asset_origin) {
				T::NonFungibles::transfer(collection_id, item_id, who)
			} else {
				T::NonFungibles::mint_into(
					collection_id,
					item_id,
					who,
					&T::ItemConfig::default(),
					false,
				)
			}
		}

		fn register_withdrawal(
			epoch: EpochNumber,
			depositor: &T::AccountId,
			asset: &AssetOf<T>,
		) -> DispatchResult {
			Withdrawals::<T>::try_mutate(depositor, asset, |maybe_entry| {
				match maybe_entry {
					None => *maybe_entry = Some((epoch, 0)),
					Some((_, chunk)) => {
						*chunk = chunk.saturating_add(1);
					},
				}

				Ok(())
			})
		}

		fn clear_old_deposit_entries(epoch: EpochNumber) -> DispatchResult {
			let epoch_set = {
				let epoch_vec = Deposits::<T>::iter_keys()
					.map(|(epoch, _, _)| epoch)
					.collect::<sp_std::vec::Vec<_>>();

				if epoch_vec.is_empty() {
					None
				} else {
					let min_epoch = *epoch_vec.iter().min().unwrap();
					let max_epoch = {
						let max = *epoch_vec.iter().max().unwrap();

						if max > epoch {
							epoch
						} else {
							max
						}
					};

					Some((min_epoch, max_epoch))
				}
			};

			if let Some((min_epoch, max_epoch)) = epoch_set {
				for i in min_epoch..=max_epoch {
					let _ = Deposits::<T>::clear_prefix((i,), 10, None);
				}
			}

			Ok(())
		}
	}
}
