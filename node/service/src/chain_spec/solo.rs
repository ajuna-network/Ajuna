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

use crate::chain_spec::{chain_spec_properties, get_well_known_accounts};
use ajuna_primitives::Balance;
use ajuna_solo_runtime::{
	currency::AJUNS, AssetsConfig, AuraConfig, BalancesConfig, CouncilConfig, GenesisConfig,
	GrandpaConfig, SudoConfig, SystemConfig, VestingConfig, WASM_BINARY,
};
use sc_service::ChainType;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

pub fn development_config(chain_type: ChainType) -> Result<ChainSpec, String> {
	let properties = chain_spec_properties("AJUN", 12, 42);
	let name = match chain_type {
		ChainType::Local => "Ajuna Local Testnet",
		ChainType::Development => "Ajuna Dev Testnet",
		_ => return Err("Call dedicated functions for other chain types.".into()),
	};
	let id = name.to_lowercase().replace(' ', "_");
	let protocol_id = name.to_lowercase().replace(' ', "-");

	Ok(ChainSpec::from_genesis(
		name,
		&id,
		chain_type,
		development_config_genesis,
		vec![],
		None,
		Some(&protocol_id),
		None,
		Some(properties),
		None,
	))
}

pub fn testnet_config() -> Result<ChainSpec, String> {
	let properties = chain_spec_properties("AJUN", 12, 42);

	Ok(ChainSpec::from_genesis(
		"Ajuna Testnet",
		"ajuna_testnet",
		ChainType::Live,
		testnet_config_genesis,
		vec![],
		None,
		Some("ajuna-testnet"),
		None,
		Some(properties),
		None,
	))
}

struct Config {
	aura: AuraConfig,
	grandpa: GrandpaConfig,
	sudo: SudoConfig,
	council: CouncilConfig,
	balances: BalancesConfig,
	assets: AssetsConfig,
	vesting: VestingConfig,
}

fn development_config_genesis() -> GenesisConfig {
	let accounts = get_well_known_accounts();
	let (aura_authorities, grandpa_authorities) = [accounts.alice_authority].into_iter().unzip();

	const INITIAL_BALANCE: Balance = 1_000_000_000 * AJUNS;
	const INITIAL_ASSET_BALANCE: Balance = 1_000_000_000;
	const VEST_BALANCE: Balance = 123 * AJUNS;
	let vest_alice_from_0_to_10_at_period_1 = (accounts.alice.clone(), 0, 1, 10, VEST_BALANCE);
	let vest_bob_from_0_to_20_at_period_2 = (accounts.bob.clone(), 0, 2, 10, VEST_BALANCE);
	let vest_charlie_from_0_to_36_at_period_3 = (accounts.charlie.clone(), 0, 3, 12, VEST_BALANCE);
	let cliff_vest_dave_at_10 = (accounts.dave.clone(), 9, 10, 1, VEST_BALANCE);
	let cliff_vest_eve_at_20 = (accounts.eve.clone(), 19, 20, 1, VEST_BALANCE);
	let cliff_vest_ferdie_at_30 = (accounts.ferdie.clone(), 29, 30, 1, VEST_BALANCE);

	compose_genesis_config(Config {
		aura: AuraConfig { authorities: aura_authorities },
		grandpa: GrandpaConfig { authorities: grandpa_authorities },
		sudo: SudoConfig { key: Some(accounts.alice.clone()) },
		council: CouncilConfig {
			members: vec![accounts.bob.clone(), accounts.charlie.clone(), accounts.dave.clone()],
			phantom: Default::default(),
		},
		balances: BalancesConfig {
			balances: vec![
				(accounts.alice.clone(), INITIAL_BALANCE),
				(accounts.bob.clone(), INITIAL_BALANCE),
				(accounts.charlie.clone(), INITIAL_BALANCE),
				(accounts.dave, VEST_BALANCE),
				(accounts.eve, VEST_BALANCE),
				(accounts.ferdie, VEST_BALANCE),
				(accounts.alice_stash, INITIAL_BALANCE),
				(accounts.bob_stash, INITIAL_BALANCE),
			],
		},
		assets: AssetsConfig {
			assets: vec![(0, accounts.alice.clone(), true, 1)],
			metadata: vec![(0, "Dotmog".into(), "DMOG".into(), 3)],
			accounts: vec![
				(0, accounts.alice, INITIAL_ASSET_BALANCE),
				(0, accounts.bob, INITIAL_ASSET_BALANCE),
				(0, accounts.charlie, INITIAL_ASSET_BALANCE),
			],
		},
		vesting: VestingConfig {
			vesting: vec![
				vest_alice_from_0_to_10_at_period_1,
				vest_bob_from_0_to_20_at_period_2,
				vest_charlie_from_0_to_36_at_period_3,
				cliff_vest_dave_at_10,
				cliff_vest_eve_at_20,
				cliff_vest_ferdie_at_30,
			],
		},
	})
}

fn testnet_config_genesis() -> GenesisConfig {
	use hex_literal::hex;
	use sp_core::crypto::UncheckedInto;

	let accounts = get_well_known_accounts();

	const INITIAL_BALANCE: Balance = 1_000_000_000 * AJUNS;

	compose_genesis_config(Config {
		aura: AuraConfig {
			authorities: vec![
				// 5GRaE4bbSBxXtMmfGsWvycRSmLE1KA1ZUmAdyKQTyhFTFEy8
				hex!["c0db660b24bcf1b717a3a3e992cdd6d76710230848e664ddb4a06c1721df7c55"]
					.unchecked_into(),
			],
		},
		grandpa: GrandpaConfig {
			authorities: vec![
				// 5EpCKebe3iTSTUBMM4mFzwEKkbJBA3CdtGiVabsPjwMAyPsd
				(
					hex!["79a3d774934ac9660dd62e32b35679456d8836d61dc8537068d0559c0f4b566f"]
						.unchecked_into(),
					1,
				),
			],
		},
		sudo: SudoConfig { key: Some(accounts.alice.clone()) },
		council: CouncilConfig::default(),
		balances: BalancesConfig {
			balances: vec![
				(accounts.alice, INITIAL_BALANCE),
				(accounts.bob, INITIAL_BALANCE),
				(accounts.charlie, INITIAL_BALANCE),
				(accounts.dave, INITIAL_BALANCE),
				(accounts.eve, INITIAL_BALANCE),
				(accounts.ferdie, INITIAL_BALANCE),
				(accounts.alice_stash, INITIAL_BALANCE),
				(accounts.bob_stash, INITIAL_BALANCE),
				(accounts.charlie_stash, INITIAL_BALANCE),
				(accounts.dave_stash, INITIAL_BALANCE),
				(accounts.eve_stash, INITIAL_BALANCE),
				(accounts.ferdie_stash, INITIAL_BALANCE),
			],
		},
		assets: AssetsConfig::default(),
		vesting: VestingConfig::default(),
	})
}

// Composes config with defaults to return initial storage state for FRAME modules.
fn compose_genesis_config(config: Config) -> GenesisConfig {
	let wasm_binary = WASM_BINARY.expect(
		"Development wasm binary is not available. Please rebuild with SKIP_WASM_BUILD disabled.",
	);
	let Config { aura, grandpa, sudo, council, balances, assets, vesting } = config;
	GenesisConfig {
		// overridden config
		aura,
		grandpa,
		sudo,
		council,
		balances,
		assets,
		vesting,
		// default config
		system: SystemConfig { code: wasm_binary.to_vec() },
		transaction_payment: Default::default(),
		council_membership: Default::default(),
		treasury: Default::default(),
		democracy: Default::default(),
		awesome_avatars: Default::default(),
		nft_staking: Default::default(),
	}
}
