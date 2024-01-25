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

	pub type AffiliatedAccountsOf<T> =
		BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::AffiliateMaxLevel>;

	pub type PayoutRuleOf<T> = PayoutRule<<T as Config>::AffiliateMaxLevel>;

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The maximum depth of the affiliate relation chain,
		#[pallet::constant]
		type AffiliateMaxLevel: Get<u32>;
	}

	/// Stores the affiliated accounts from the perspectives of the affiliatee
	#[pallet::storage]
	#[pallet::getter(fn affiliatees)]
	pub type Affiliatees<T: Config> =
		StorageMap<_, Identity, T::AccountId, AffiliatedAccountsOf<T>, OptionQuery>;

	/// Store affiliators aka accounts that have affilatees and earn rewards from them.
	/// Such accounts can't be affiliatees anymore.
	#[pallet::storage]
	#[pallet::getter(fn affiliators)]
	pub type Affiliators<T: Config> =
		StorageMap<_, Identity, T::AccountId, AffiliatorState, ValueQuery>;

	/// Stores the affiliate logic rules
	#[pallet::storage]
	pub type AffiliateRules<T: Config> =
		StorageMap<_, Blake2_128Concat, RuleId, PayoutRuleOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
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
	pub enum Error<T> {
		/// There is no account set as the organizer
		OrganizerNotSet,
		/// An account cannot affiliate itself
		CannotAffiliateSelf,
		/// The account is not allowed to receive affiliates
		CannotAffiliateToAccount,
		/// This account has reached the affiliate limit
		CannotAffiliateMoreAccounts,
		/// This account has already been affiliated by another affiliator
		CannotAffiliateAlreadyAffiliatedAccount,
		/// This account is already an affiliator, so it cannot affiliate to another account
		CannotAffiliateExistingAffiliator,
		/// The account is blocked, so it cannot be affiliated to
		CannotAffiliateBlocked,
		/// The given extrinsic identifier is already paired with an affiliate rule
		ExtrinsicAlreadyHasRule,
	}

	impl<T: Config> Pallet<T> {
		fn add_new_affiliate_to(
			affiliator: T::AccountId,
			affiliatee: T::AccountId,
		) -> DispatchResult {
			let accounts = {
				let mut accounts_vec = Affiliatees::<T>::take(&affiliator).unwrap_or_default();

				Self::try_add_account_to(&mut accounts_vec, affiliator.clone())?;

				accounts_vec
			};

			Affiliatees::<T>::insert(affiliatee, accounts);
			Affiliators::<T>::try_mutate(&affiliator, |state| {
				state.affiliates = state
					.affiliates
					.checked_add(1)
					.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

				Ok(())
			})
		}

		fn try_add_account_to(
			accounts: &mut AffiliatedAccountsOf<T>,
			account: T::AccountId,
		) -> DispatchResult {
			if accounts.len() == T::AffiliateMaxLevel::get() as usize {
				accounts.pop();
			}
			accounts
				.try_insert(0, account)
				.map_err(|_| Error::<T>::CannotAffiliateMoreAccounts.into())
		}
	}

	impl<T: Config> AffiliateInspector<AccountIdOf<T>> for Pallet<T> {
		fn get_affiliator_chain_for(account: &AccountIdOf<T>) -> Option<Vec<AccountIdOf<T>>> {
			Affiliatees::<T>::get(account).map(|accounts| accounts.into_inner())
		}

		fn get_affiliate_count_for(account: &AccountIdOf<T>) -> u32 {
			Affiliators::<T>::get(account).affiliates
		}
	}

	impl<T: Config> AffiliateMutator<AccountIdOf<T>> for Pallet<T> {
		fn try_mark_account_as_affiliatable(account: &AccountIdOf<T>) -> DispatchResult {
			Affiliators::<T>::try_mutate(account, |state| {
				ensure!(
					state.status != AffiliatableStatus::Blocked,
					Error::<T>::CannotAffiliateBlocked
				);

				state.status = AffiliatableStatus::Affiliatable;

				Ok(())
			})
		}

		fn force_mark_account_as_affiliatable(account: &AccountIdOf<T>) {
			Affiliators::<T>::mutate(account, |state| {
				state.status = AffiliatableStatus::Affiliatable;
			});
		}

		fn mark_account_as_blocked(account: &AccountIdOf<T>) {
			Affiliators::<T>::mutate(account, |state| {
				state.status = AffiliatableStatus::Blocked;
			});
		}

		fn try_add_affiliate_to(
			account: &AccountIdOf<T>,
			affiliate: &AccountIdOf<T>,
		) -> DispatchResult {
			ensure!(account != affiliate, Error::<T>::CannotAffiliateSelf);

			let affiliate_state = Affiliators::<T>::get(affiliate);
			ensure!(affiliate_state.affiliates == 0, Error::<T>::CannotAffiliateExistingAffiliator);

			ensure!(
				!Affiliatees::<T>::contains_key(affiliate),
				Error::<T>::CannotAffiliateAlreadyAffiliatedAccount
			);

			let affiliator_state = Affiliators::<T>::get(account);
			ensure!(
				affiliator_state.status == AffiliatableStatus::Affiliatable,
				Error::<T>::CannotAffiliateToAccount
			);

			Self::add_new_affiliate_to(account.clone(), affiliate.clone())?;

			Self::deposit_event(Event::AccountAffiliated {
				account: affiliate.clone(),
				to: account.clone(),
			});

			Ok(())
		}

		fn try_clear_affiliation_for(account: &AccountIdOf<T>) -> DispatchResult {
			Affiliatees::<T>::take(account)
				.and_then(|mut affiliate_chain| affiliate_chain.pop())
				.map_or_else(
					|| Ok(()),
					|affiliator| {
						Affiliators::<T>::try_mutate(&affiliator, |state| {
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

	impl<T: Config> RuleInspector<T::AffiliateMaxLevel> for Pallet<T> {
		fn get_rule_for(rule_id: RuleId) -> Option<PayoutRuleOf<T>> {
			AffiliateRules::<T>::get(rule_id)
		}
	}

	impl<T: Config> RuleMutator<AccountIdOf<T>, T::AffiliateMaxLevel> for Pallet<T> {
		fn try_add_rule_for(rule_id: RuleId, rule: PayoutRuleOf<T>) -> DispatchResult {
			ensure!(
				!AffiliateRules::<T>::contains_key(rule_id),
				Error::<T>::ExtrinsicAlreadyHasRule
			);
			AffiliateRules::<T>::insert(rule_id, rule);
			Self::deposit_event(Event::RuleAdded { rule_id });

			Ok(())
		}

		fn clear_rule_for(rule_id: RuleId) {
			AffiliateRules::<T>::remove(rule_id);

			Self::deposit_event(Event::RuleCleared { rule_id });
		}
	}
}
