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
