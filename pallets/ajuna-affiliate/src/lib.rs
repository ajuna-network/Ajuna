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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod traits;

use frame_support::pallet_prelude::*;

use traits::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use sp_runtime::ArithmeticError;

	pub type AffiliatedAccountsOf<T, I> =
		BoundedVec<<T as frame_system::Config>::AccountId, <T as Config<I>>::AffiliateMaxLevel>;

	pub type PayoutRuleFor<T, I> = PayoutRule<<T as Config<I>>::AffiliateMaxLevel>;

	pub type AccountIdFor<T> = <T as frame_system::Config>::AccountId;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The maximum depth of the affiliate relation chain,
		#[pallet::constant]
		type AffiliateMaxLevel: Get<u32>;
	}

	/// Stores the affiliated accounts from the perspectives of the affiliatee
	#[pallet::storage]
	#[pallet::getter(fn affiliatees)]
	pub type Affiliatees<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::AccountId, AffiliatedAccountsOf<T, I>, OptionQuery>;

	/// Store affiliators aka accounts that have affilatees and earn rewards from them.
	/// Such accounts can't be affiliatees anymore.
	#[pallet::storage]
	#[pallet::getter(fn affiliators)]
	pub type Affiliators<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::AccountId, AffiliatorState, ValueQuery>;

	/// Stores the affiliate logic rules
	#[pallet::storage]
	pub type AffiliateRules<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, RuleId, PayoutRuleFor<T, I>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// An organizer has been set.
		OrganizerSet {
			organizer: T::AccountId,
		},
		AccountAffiliated {
			account: T::AccountId,
			to: T::AccountId,
		},
		RuleAdded {
			rule_id: RuleId,
		},
		RuleCleared {
			rule_id: RuleId,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// There is no account set as the organizer
		OrganizerNotSet,
		/// An account cannot affiliate itself
		CannotAffiliateSelf,
		/// The account is not allowed to receive affiliates
		TargetAccountIsNotAffiliatable,
		/// This account has reached the affiliate limit
		CannotAffiliateMoreAccounts,
		/// This account has already been affiliated by another affiliator
		CannotAffiliateAlreadyAffiliatedAccount,
		/// This account is already an affiliator, so it cannot affiliate to another account
		CannotAffiliateToExistingAffiliator,
		/// The account is blocked, so it cannot be affiliated to
		CannotAffiliateBlocked,
		/// The given extrinsic identifier is already paired with an affiliate rule
		ExtrinsicAlreadyHasRule,
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn add_new_affiliate_to(
			affiliator: T::AccountId,
			affiliatee: T::AccountId,
		) -> DispatchResult {
			let mut accounts = Affiliatees::<T, I>::take(&affiliator).unwrap_or_default();

			Self::try_add_account_to(&mut accounts, affiliator.clone())?;

			Affiliatees::<T, I>::insert(affiliatee, accounts);
			Affiliators::<T, I>::try_mutate(&affiliator, |state| {
				state.affiliates = state
					.affiliates
					.checked_add(1)
					.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

				Ok(())
			})
		}

		fn try_add_account_to(
			accounts: &mut AffiliatedAccountsOf<T, I>,
			account: T::AccountId,
		) -> DispatchResult {
			if accounts.len() == T::AffiliateMaxLevel::get() as usize {
				accounts.pop();
			}
			accounts
				.try_insert(0, account)
				.map_err(|_| Error::<T, I>::CannotAffiliateMoreAccounts.into())
		}
	}

	impl<T: Config<I>, I: 'static> AffiliateInspector<AccountIdFor<T>> for Pallet<T, I> {
		fn get_affiliator_chain_for(account: &AccountIdFor<T>) -> Option<Vec<AccountIdFor<T>>> {
			Affiliatees::<T, I>::get(account).map(|accounts| accounts.into_inner())
		}

		fn get_affiliate_count_for(account: &AccountIdFor<T>) -> u32 {
			Affiliators::<T, I>::get(account).affiliates
		}
	}

	impl<T: Config<I>, I: 'static> AffiliateMutator<AccountIdFor<T>> for Pallet<T, I> {
		fn try_mark_account_as_affiliatable(account: &AccountIdFor<T>) -> DispatchResult {
			Affiliators::<T, I>::try_mutate(account, |state| {
				ensure!(
					state.status != AffiliatableStatus::Blocked,
					Error::<T, I>::CannotAffiliateBlocked
				);

				state.status = AffiliatableStatus::Affiliatable;

				Ok(())
			})
		}

		fn force_mark_account_as_affiliatable(account: &AccountIdFor<T>) {
			Affiliators::<T, I>::mutate(account, |state| {
				state.status = AffiliatableStatus::Affiliatable;
			});
		}

		fn mark_account_as_blocked(account: &AccountIdFor<T>) {
			Affiliators::<T, I>::mutate(account, |state| {
				state.status = AffiliatableStatus::Blocked;
			});
		}

		fn try_add_affiliate_to(
			account: &AccountIdFor<T>,
			affiliate: &AccountIdFor<T>,
		) -> DispatchResult {
			ensure!(account != affiliate, Error::<T, I>::CannotAffiliateSelf);

			let affiliate_state = Affiliators::<T, I>::get(affiliate);
			ensure!(
				affiliate_state.affiliates == 0,
				Error::<T, I>::CannotAffiliateToExistingAffiliator
			);

			ensure!(
				!Affiliatees::<T, I>::contains_key(affiliate),
				Error::<T, I>::CannotAffiliateAlreadyAffiliatedAccount
			);

			let affiliator_state = Affiliators::<T, I>::get(account);
			ensure!(
				affiliator_state.status == AffiliatableStatus::Affiliatable,
				Error::<T, I>::TargetAccountIsNotAffiliatable
			);

			Self::add_new_affiliate_to(account.clone(), affiliate.clone())?;

			Self::deposit_event(Event::AccountAffiliated {
				account: affiliate.clone(),
				to: account.clone(),
			});

			Ok(())
		}

		fn try_clear_affiliation_for(account: &AccountIdFor<T>) -> DispatchResult {
			Affiliatees::<T, I>::take(account)
				.and_then(|mut affiliate_chain| affiliate_chain.pop())
				.map_or_else(
					|| Ok(()),
					|affiliator| {
						Affiliators::<T, I>::try_mutate(&affiliator, |state| {
							state.affiliates = state
								.affiliates
								.checked_sub(1)
								.ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;

							Ok(())
						})
					},
				)
		}
	}

	impl<T: Config<I>, I: 'static> RuleInspector<T::AffiliateMaxLevel> for Pallet<T, I> {
		fn get_rule_for(rule_id: RuleId) -> Option<PayoutRuleFor<T, I>> {
			AffiliateRules::<T, I>::get(rule_id)
		}
	}

	impl<T: Config<I>, I: 'static> RuleMutator<AccountIdFor<T>, T::AffiliateMaxLevel> for Pallet<T, I> {
		fn try_add_rule_for(rule_id: RuleId, rule: PayoutRuleFor<T, I>) -> DispatchResult {
			ensure!(
				!AffiliateRules::<T, I>::contains_key(rule_id),
				Error::<T, I>::ExtrinsicAlreadyHasRule
			);
			AffiliateRules::<T, I>::insert(rule_id, rule);
			Self::deposit_event(Event::RuleAdded { rule_id });

			Ok(())
		}

		fn clear_rule_for(rule_id: RuleId) {
			AffiliateRules::<T, I>::remove(rule_id);

			Self::deposit_event(Event::RuleCleared { rule_id });
		}
	}
}
