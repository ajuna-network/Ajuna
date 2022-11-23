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

use codec::{Decode, Encode};
use frame_support::{pallet_prelude::MaxEncodedLen, traits::InstanceFilter, RuntimeDebug};
use scale_info::TypeInfo;

use crate::RuntimeCall;

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo,
)]
pub enum ProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
	IdentityJudgement,
	CancelProxy,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				RuntimeCall::System(..) |
				RuntimeCall::Scheduler(..) |
				RuntimeCall::Timestamp(..) |
				// Specifically omitting the entire Balances pallet
				RuntimeCall::Authorship(..) |
				RuntimeCall::Session(..) |
				RuntimeCall::Council(..) |
				RuntimeCall::CouncilMembership(..) |
				RuntimeCall::Treasury(..) |
				RuntimeCall::Vesting(orml_vesting::Call::claim{..}) |
				RuntimeCall::Vesting(orml_vesting::Call::claim_for{..}) |
				// Specifically omitting Vesting `vested_transfer`, and `update_vesting_schedules`
				RuntimeCall::Utility(..) |
				RuntimeCall::Identity(..) |
				RuntimeCall::Proxy(..) |
				RuntimeCall::Multisig(..)
			),
			ProxyType::Governance => {
				matches!(
					c,
					RuntimeCall::Council(..) | RuntimeCall::Treasury(..) | RuntimeCall::Utility(..)
				)
			},
			ProxyType::Staking => {
				matches!(c, RuntimeCall::Session(..) | RuntimeCall::Utility(..))
			},
			ProxyType::IdentityJudgement => matches!(
				c,
				RuntimeCall::Identity(pallet_identity::Call::provide_judgement { .. }) |
					RuntimeCall::Utility(..)
			),
			ProxyType::CancelProxy => {
				matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. }))
			},
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}
