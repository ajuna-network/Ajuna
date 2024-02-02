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

	/// Attempts to add an affiliate link between affiliate and account
	fn try_add_affiliate_to(account: &AccountId, affiliate: &AccountId) -> DispatchResult;

	/// Attempts to remove the affiliate link from account
	fn try_clear_affiliation_for(account: &AccountId) -> DispatchResult;
}

pub trait RuleInspector<RuleId, Rule> {
	/// Gets the rule data for a given 'extrinsic_id' mapped rule, or
	/// None if no rule is associated with the given 'extrinsic_id'
	fn get_rule_for(rule_id: RuleId) -> Option<Rule>;
}

pub trait RuleMutator<RuleId, Rule> {
	/// Tries to add a rule for 'extrinsic_id', fails to do so
	/// if there's already a rule present.
	fn try_add_rule_for(rule_id: RuleId, rule: Rule) -> DispatchResult;

	/// Removes the rule mapping for 'extrinsic_id'
	fn clear_rule_for(rule_id: RuleId);
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
	pub status: AffiliatableStatus,
	pub affiliates: u32,
}
