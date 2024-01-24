use frame_support::pallet_prelude::*;

pub trait AffiliateInspector<AccountId> {
	/// Returns a vector of accounts that 'account' is affiliated to.
	///
	/// The latest account in the vector is the direct affiliate while the others,
	/// are indirect affiliates.
	///
	/// If the account is not affiliated to any other account, returns None.
	fn get_affiliator_chain_for(account: &AccountId) -> Option<Vec<AccountId>>;

	/// Returns the number of accounts that are affiliated with 'account'.
	fn get_affiliate_count_for(account: &AccountId) -> u32;
}

pub trait AffiliateMutator<AccountId> {
	/// Tries to mark an account as [AffiliatableStatus::Affiliatable], fails
	/// to do so if the account is in the [AffiliatableStatus::Blocked] state.
	fn try_mark_account_as_affiliatable(account: &AccountId) -> DispatchResult;

	/// Forces the marking on an account as [AffiliatableStatus::Affiliatable] ignoring
	/// its current state.
	fn force_mark_account_as_affiliatable(account: &AccountId);

	/// Marks an account as [AffiliatableStatus::Blocked]
	fn mark_account_as_blocked(account: &AccountId);

	fn try_add_affiliate_to(account: &AccountId, affiliate: &AccountId) -> DispatchResult;

	fn clear_affiliation_for(account: &AccountId);
}

pub type PalletIndex = u32;
pub type CallIndex = u8;

pub type ExtrinsicId = (PalletIndex, CallIndex);

pub trait RuleInspector {
	/// Gets the data for a given 'extrinsic_id' mapped rule, or
	/// None if no rule is associated with the given 'extrinsic_id'
	fn get_rule_for(extrinsic_id: ExtrinsicId) -> Option<u8>;
}

pub trait RuleMutator<AccountId> {
	/// Tries to add a rule for 'extrinsic_id', fails to do so
	/// if there's already a rule present.
	fn try_add_rule_for(extrinsic_id: ExtrinsicId, rule: u8) -> DispatchResult;

	/// Removes the rule mapping for 'extrinsic_id'
	fn clear_rule_for(extrinsic_id: ExtrinsicId);

	/// Tries to execute the rule associated with 'extrinsic_id', will fail
	/// if no rule is associated with it, or if rule payout is not possible.
	fn try_execute_rule_for(extrinsic_id: ExtrinsicId, account: &AccountId) -> DispatchResult;
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, Copy, Clone, PartialEq)]
pub enum AffiliatableStatus {
	#[default]
	NonAffiliatable,
	Affiliatable,
	Blocked,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, Copy, Clone, PartialEq)]
pub struct AffiliatorState {
	pub(crate) status: AffiliatableStatus,
	pub(crate) affiliates: u32,
}
