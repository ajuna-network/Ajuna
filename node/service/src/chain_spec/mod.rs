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

use ajuna_primitives::{AccountId, AccountPublic};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::IdentifyAccount;

#[cfg(feature = "solo")]
pub mod solo;

#[cfg(feature = "bajun")]
pub mod bajun;

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_public_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_public_from_seed::<TPublic>(seed)).into_account()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AuraId = sp_consensus_aura::sr25519::AuthorityId;
type GrandpaId = sp_finality_grandpa::AuthorityId;
type GrandpaWeight = sp_finality_grandpa::AuthorityWeight;
pub type AuthorityPublicKey = (AuraId, (GrandpaId, GrandpaWeight));

// Generate an authority key.
fn authority_keys_from_seed(s: &str) -> AuthorityPublicKey {
	(get_public_from_seed::<AuraId>(s), (get_public_from_seed::<GrandpaId>(s), 1))
}

pub struct WellKnownAccounts {
	// accounts
	pub alice: AccountId,
	pub bob: AccountId,
	pub charlie: AccountId,
	pub dave: AccountId,
	pub eve: AccountId,
	pub ferdie: AccountId,

	// stashes
	pub alice_stash: AccountId,
	pub bob_stash: AccountId,
	pub charlie_stash: AccountId,
	pub dave_stash: AccountId,
	pub eve_stash: AccountId,
	pub ferdie_stash: AccountId,

	// authorities
	pub alice_authority: AuthorityPublicKey,
	pub bob_authority: AuthorityPublicKey,
	pub charlie_authority: AuthorityPublicKey,
}

pub fn get_well_known_accounts() -> WellKnownAccounts {
	WellKnownAccounts {
		alice: get_account_id_from_seed::<sr25519::Public>("Alice"),
		bob: get_account_id_from_seed::<sr25519::Public>("Bob"),
		charlie: get_account_id_from_seed::<sr25519::Public>("Charlie"),
		dave: get_account_id_from_seed::<sr25519::Public>("Dave"),
		eve: get_account_id_from_seed::<sr25519::Public>("Eve"),
		ferdie: get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		alice_stash: get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		bob_stash: get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		charlie_stash: get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
		dave_stash: get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
		eve_stash: get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
		ferdie_stash: get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		alice_authority: authority_keys_from_seed("Alice"),
		bob_authority: authority_keys_from_seed("Bob"),
		charlie_authority: authority_keys_from_seed("Charlie"),
	}
}

// Configure chain specification metadata properties.
pub fn chain_spec_properties(
	symbol: &str,
	decimal: u16,
	address_prefix: u16,
) -> sc_chain_spec::Properties {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), symbol.into());
	properties.insert("tokenDecimals".into(), decimal.into());
	properties.insert("ss58Format".into(), address_prefix.into());
	properties
}
