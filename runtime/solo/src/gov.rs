use crate::{
	BlockWeights, OriginCaller, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, AJUNS, DAYS,
};
use ajuna_primitives::{AccountId, Balance, BlockNumber};
use frame_support::{
	dispatch::RawOrigin,
	parameter_types,
	traits::{ConstBool, ConstU32, EitherOfDiverse, EnsureOrigin},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_collective::{EnsureMember, EnsureProportionAtLeast, EnsureProportionMoreThan};
use sp_runtime::Perbill;
use sp_std::marker::PhantomData;

pub type EnsureRootOrMoreThanHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;

pub type EnsureRootOrMoreThanHalfTechnicalCommittee = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 1, 2>,
>;

pub type EnsureRootOrAllTechnicalCommittee = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 1, 1>,
>;

type AccountIdFor<T> = <T as frame_system::Config>::AccountId;

/// Ensures that the signer of a transaction is a member of said collective instance.
///
/// This is fundamentally different from the `pallet_collective::EnsureMember`,
/// which checks if the referendum to be executed has been ayed by any member.
/// It is a different kind of origin.
pub struct EnsureSignerIsCollectiveMember<T, I: 'static>(PhantomData<(T, I)>);
impl<
		O: Into<Result<RawOrigin<AccountIdFor<T>>, O>> + From<RawOrigin<AccountIdFor<T>>>,
		T: pallet_collective::Config<I>,
		I,
	> EnsureOrigin<O> for EnsureSignerIsCollectiveMember<T, I>
{
	type Success = AccountIdFor<T>;
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			RawOrigin::Signed(a) if pallet_collective::Pallet::<T, I>::is_member(&a) => Ok(a),
			r => Err(O::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		use parity_scale_codec::Decode;
		use sp_runtime::traits::TrailingZeroInput;
		let zero_account_id =
			<AccountIdFor<T>>::decode(&mut TrailingZeroInput::zeroes()).map_err(|_| ())?;
		Ok(O::from(RawOrigin::Signed(zero_account_id)))
	}
}

/// Council collective instance declaration.
///
/// The council primarily serves to optimize and balance the inclusive referendum system,
/// by being allowed to propose external democracy proposals, which can be fast tracked and
/// bypass the one active referendum at a time rule.
///
/// It also controls the treasury.
type CouncilCollective = pallet_collective::Instance1;

parameter_types! {
	pub CouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub MaxProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
}

impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = ConstU32<100>;
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRootOrMoreThanHalfCouncil;
	type MaxProposalWeight = MaxProposalWeight;
}

/// The technical committee primarily serves to safeguard against malicious referenda,
/// and fast track critical referenda.
pub type TechnicalCommitteeInstance = pallet_collective::Instance2;

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 3 * DAYS;
}

impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = ConstU32<100>;
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRootOrMoreThanHalfCouncil;
	type MaxProposalWeight = MaxProposalWeight;
}

parameter_types! {
	pub const ThreeDays: BlockNumber = 3 * DAYS;
	pub const TwentyEightDays: BlockNumber = 28 * DAYS;
	pub const ThirtyDays: BlockNumber = 30 * DAYS;
	pub EnactmentPeriod: BlockNumber = 7 * DAYS;
	pub const MinimumDeposit: Balance = AJUNS;
}

impl pallet_democracy::Config for Runtime {
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = pallet_scheduler::Pallet<Runtime>;
	type Preimages = pallet_preimage::Pallet<Runtime>;
	type Currency = pallet_balances::Pallet<Runtime>;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = TwentyEightDays;
	type VotingPeriod = TwentyEightDays;
	type VoteLockingPeriod = EnactmentPeriod;
	type MinimumDeposit = MinimumDeposit;
	type InstantAllowed = ConstBool<true>;
	type FastTrackVotingPeriod = ThreeDays;
	type CooloffPeriod = TwentyEightDays;
	type MaxVotes = ConstU32<100>;
	type MaxProposals = ConstU32<100>;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;
	type ExternalOrigin = EnsureRootOrMoreThanHalfCouncil;
	type ExternalMajorityOrigin = EnsureRootOrMoreThanHalfCouncil;
	type ExternalDefaultOrigin = EnsureRootOrMoreThanHalfCouncil;
	// Initially, we want that only the council can submit proposals to
	// prevent malicious proposals.
	type SubmitOrigin = EnsureSignerIsCollectiveMember<Runtime, CouncilCollective>;
	type FastTrackOrigin = EnsureRootOrMoreThanHalfTechnicalCommittee;
	type InstantOrigin = EnsureRootOrMoreThanHalfTechnicalCommittee;
	// To cancel a proposal that has passed.
	type CancellationOrigin = EnsureRoot<AccountId>;
	type BlacklistOrigin = EnsureRootOrMoreThanHalfCouncil;
	// To cancel a proposal before it has passed, and slash its backers.
	type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = EnsureMember<AccountId, TechnicalCommitteeInstance>;
	type PalletsOrigin = OriginCaller;
	type Slash = pallet_treasury::Pallet<Runtime>;
}
