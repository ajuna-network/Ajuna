use crate::{BlockWeights, OriginCaller, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, DAYS};
use ajuna_primitives::{AccountId, Balance, BlockNumber};
use frame_support::{
	parameter_types,
	traits::{ConstBool, ConstU32, EitherOfDiverse},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_collective::{EnsureProportionAtLeast, EnsureProportionMoreThan};
use sp_runtime::Perbill;

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

/// Council collective instance declaration.
///
/// The council primarily serves to optimize and balance the inclusive referendum system,
/// by being allowed to propose external democracy proposals, which can be fast tracked and
/// bypass the one active referendum at a time rule.
///
/// It also control the treasury.
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
	type DefaultVote = pallet_collective::PrimeDefaultVote;
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
	// The maximum amount of time (in blocks) for technical committee members to vote on motions.
	// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = ConstU32<100>;
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRootOrMoreThanHalfCouncil;
	type MaxProposalWeight = MaxProposalWeight;
}

parameter_types! {
	pub const ThirtyDays: BlockNumber = 30 * DAYS;
	pub const TwentyEightDays: BlockNumber = 28 * DAYS;
	pub const ThreeDays: BlockNumber = 3 * DAYS;
	pub const MinimumDeposit: Balance = 1;
	pub EnactmentPeriod: BlockNumber = 7 * DAYS;
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
	type SubmitOrigin = EnsureSigned<AccountId>;
	type FastTrackOrigin = EnsureRootOrMoreThanHalfTechnicalCommittee;
	type InstantOrigin = EnsureRootOrMoreThanHalfTechnicalCommittee;
	// To cancel a proposal that has passed.
	type CancellationOrigin = EnsureRoot<AccountId>;
	type BlacklistOrigin = EnsureRootOrMoreThanHalfCouncil;
	// To cancel a proposal before it has passed, and slash its backers.
	type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCommitteeInstance>;
	type PalletsOrigin = OriginCaller;
	type Slash = pallet_treasury::Pallet<Runtime>;
}
