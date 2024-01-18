use frame_system::Config;
use sp_runtime::DispatchError;

pub type PalletIndex = u32;
pub type CallIndex = u8;

pub type ExtrinsicId = (PalletIndex, CallIndex);

pub trait AffiliateHandler<AccountId> {
	fn get_affiliator_for(account: AccountId) -> Option<AccountId>;

	fn has_rule_for(extrinsic_id: ExtrinsicId) -> bool;

	fn execute_rule_for(extrinsic_id: ExtrinsicId, account: AccountId)
		-> Result<(), DispatchError>;
}
