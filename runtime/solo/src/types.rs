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

pub mod governance {
	use crate::CouncilCollective;
	use ajuna_primitives::AccountId;
	use frame_support::traits::EnsureOneOf;
	use frame_system::EnsureRoot;
	use pallet_collective::{EnsureProportionAtLeast, EnsureProportionMoreThan};

	pub(crate) type EnsureRootOrMoreThanHalfCouncil = EnsureOneOf<
		EnsureRoot<AccountId>,
		EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
	pub(crate) type EnsureRootOrAtLeastTwoThirdsCouncil = EnsureOneOf<
		EnsureRoot<AccountId>,
		EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>,
	>;

	pub(crate) type EnsureAtLeastHalfCouncil =
		EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
	pub(crate) type EnsureAtLeastThreeFourthsCouncil =
		EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
	pub(crate) type EnsureAllCouncil = EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
}

pub mod proxy {
	use codec::{Decode, Encode};
	use frame_support::{pallet_prelude::MaxEncodedLen, traits::InstanceFilter, RuntimeDebug};
	use scale_info::TypeInfo;

	/// Proxy type enum lists the type of calls that are supported by the proxy
	/// pallet
	#[derive(
		Copy,
		Clone,
		Eq,
		PartialEq,
		Ord,
		PartialOrd,
		MaxEncodedLen,
		Decode,
		Encode,
		RuntimeDebug,
		TypeInfo,
	)]
	pub enum ProxyType {
		Any,
	}

	impl Default for ProxyType {
		fn default() -> Self {
			Self::Any
		}
	}

	impl<Call> InstanceFilter<Call> for ProxyType {
		fn filter(&self, _c: &Call) -> bool {
			match self {
				ProxyType::Any => true,
			}
		}
		fn is_superset(&self, o: &Self) -> bool {
			self == &ProxyType::Any || self == o
		}
	}
}
