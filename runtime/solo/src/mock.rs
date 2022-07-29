// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use frame_support::{sp_io, traits::GenesisBuild};

use crate::keyring::*;

use super::*;

pub struct ExtBuilder {
	existential_deposit: Balance,
	vesting_genesis_config: Option<Vec<(AccountId, u32, u32, u32, Balance)>>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { existential_deposit: EXISTENTIAL_DEPOSIT, vesting_genesis_config: None }
	}
}

impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: Balance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}

	pub fn vesting_genesis_config(
		mut self,
		config: Vec<(AccountId, u32, u32, u32, Balance)>,
	) -> Self {
		self.vesting_genesis_config = Some(config);
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![
				(alice(), 10_000 * self.existential_deposit),
				(bob(), 20_000 * self.existential_deposit),
				(charlie(), 30_000 * self.existential_deposit),
				(dave(), 40_000 * self.existential_deposit),
				(eve(), 10_000 * self.existential_deposit),
				(ferdie(), 9_999_000 * self.existential_deposit),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let vesting = if let Some(vesting_config) = self.vesting_genesis_config {
			vesting_config
		} else {
			vec![
				(alice(), 0, 1, 1_000, 5 * self.existential_deposit),
				(bob(), 10, 1, 2_000, 3 * self.existential_deposit),
				(charlie(), 10, 100, 2_000, 5 * self.existential_deposit),
			]
		};

		orml_vesting::GenesisConfig::<Runtime> { vesting }
			.assimilate_storage(&mut t)
			.unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
