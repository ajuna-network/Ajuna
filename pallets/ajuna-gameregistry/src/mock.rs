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

use super::*;
use crate as pallet_ajuna_gameregistry;

use frame_support::{
	construct_runtime, parameter_types, traits::EqualPrivilegeOnly, weights::Weight,
};

use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage, Perbill,
};
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const TEE_ID: u64 = 7;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Registry: pallet_ajuna_gameregistry::{Pallet, Call, Storage, Event<T>},
		Observers: pallet_membership,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1_000_000);
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
	pub const NoPreimagePostponement: Option<u64> = Some(2);
}

impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<u64>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = ();
	type NoPreimagePostponement = NoPreimagePostponement;
}

pub type GameId = u32;
ajuna_common::impl_mock_matchmaker!(u64);
ajuna_common::impl_mock_runner!(GameId);

parameter_types! {
	pub MaxAcknowledgeBatch : u32 = 2;
}

impl pallet_ajuna_gameregistry::Config for Test {
	type Proposal = Call;
	type Event = Event;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MatchMaker = MockMatchMaker;
	type Observers = Observers;
	type GameId = GameId;
	type Runner = MockRunner;
	type GetIdentifier = MockGetIdentifier;
	type MaxAcknowledgeBatch = MaxAcknowledgeBatch;
	type ShardIdentifier = H256;
	type WeightInfo = ();
}

type EnsureSignedByAccount = EnsureSigned<<Test as frame_system::Config>::AccountId>;
impl pallet_membership::Config for Test {
	type Event = Event;
	type AddOrigin = EnsureSignedByAccount;
	type RemoveOrigin = EnsureSignedByAccount;
	type SwapOrigin = EnsureSignedByAccount;
	type ResetOrigin = EnsureSignedByAccount;
	type PrimeOrigin = EnsureSignedByAccount;
	type MembershipInitialized = ();
	type MembershipChanged = ();
	type MaxMembers = frame_support::pallet_prelude::ConstU32<10>;
	type WeightInfo = ();
}

/// Build genesis storage according to the mock runtime.
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let t = GenesisConfig {
		system: Default::default(),
		observers: pallet_membership::GenesisConfig::<Test> {
			members: vec![TEE_ID],
			..Default::default()
		},
	}
	.build_storage()
	.unwrap();
	t.into()
}
