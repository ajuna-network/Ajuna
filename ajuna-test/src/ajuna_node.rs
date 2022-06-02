use crate::{
	impl_block_numbers,
	traits::{BlockProcessing, RuntimeBuilding},
};
use ajuna_solo_runtime::{AccountId, BlockNumber, Runtime, System};
use frame_support::traits::GenesisBuild;
use sp_runtime::Storage;

pub struct AjunaNode {
	/// The account owning the node(sudo)
	account_id: AccountId,
	players: Vec<AccountId>,
	sidechain: AccountId,
}

use ajuna_solo_runtime::{ObserversConfig, SudoConfig};
use sp_runtime::BuildStorage;

impl_block_numbers!(System, BlockNumber);
impl RuntimeBuilding<Runtime, BlockNumber, RuntimeBlocks> for AjunaNode {
	fn configure_storages(&self, storage: &mut Storage) {
		ajuna_solo_runtime::GenesisConfig {
			sudo: SudoConfig { key: Some(self.account_id.clone()) },
			observers: ObserversConfig {
				members: vec![self.sidechain.clone()],
				..Default::default()
			},
			..Default::default()
		}
		.assimilate_storage(storage)
		.unwrap();
	}
}

impl Default for AjunaNode {
	fn default() -> Self {
		Self { account_id: [0x0; 32].into(), players: vec![], sidechain: [0x0; 32].into() }
	}
}

impl AjunaNode {
	pub fn account(mut self, account_id: AccountId) -> Self {
		self.account_id = account_id;
		self
	}

	pub fn players(mut self, players: Vec<AccountId>) -> Self {
		self.players = players;
		self
	}

	pub fn sidechain(mut self, sidechain: AccountId) -> Self {
		self.sidechain = sidechain;
		self
	}
}

impl BlockProcessing<BlockNumber, RuntimeBlocks> for AjunaNode {
	fn on_block() {}
}
