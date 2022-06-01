use crate::{
	constants::{BlockProcessing, RuntimeBuilding},
	impl_block_numbers,
};
use ajuna_solo_runtime::{currency::MILLI_AJUNS, AccountId, BlockNumber, Runtime, System, ObserverInstance};
use frame_support::traits::GenesisBuild;
use sp_runtime::Storage;

pub struct AjunaNode {
	/// The account owning the node(sudo)
	account_id: AccountId,
	players: Vec<AccountId>,
	sidechain: AccountId,
}

impl_block_numbers!(System, BlockNumber);
impl RuntimeBuilding<Runtime, BlockNumber, RuntimeBlocks> for AjunaNode {
	fn configure_storages(&self, storage: &mut Storage) {
		pallet_membership::GenesisConfig::<Runtime, ObserverInstance> {
			members: vec![self.sidechain.clone()],
			phantom: Default::default(),
		}
			.assimilate_storage(storage)
			.unwrap();

		// Give all accounts the same balance
		let mut accounts = self.players.clone();
		accounts.push(self.account_id.clone());
		pallet_balances::GenesisConfig::<Runtime> {
			balances: accounts.iter().map(|player| (player.clone(), MILLI_AJUNS)).collect(),
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
