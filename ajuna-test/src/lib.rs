use ajuna_solo_runtime::AccountId;
use frame_support::{assert_ok};

mod ajuna_node;
mod constants;
mod sidechain;

use crate::{
	ajuna_node::AjunaNode,
	constants::{BlockProcessing, RuntimeBuilding, SIDECHAIN_SIGNING_KEY},
	sidechain::{AjunaBoard, Guess, SideChain, SigningKey},
};
use ajuna_solo_runtime::{GameRegistry, Origin};

struct Player {
	account_id: AccountId,
}

impl Player {
	pub fn queue(&self) {
		assert_ok!(GameRegistry::queue(Origin::signed(self.account_id.clone())));
	}
	pub fn play_turn(&self, guess: Guess) {
		assert_ok!(AjunaBoard::play_turn(
			sidechain::Origin::signed(self.account_id.clone()),
			guess
		));
	}
}

struct Network {}

struct SideChainSigningKey;

impl SigningKey for SideChainSigningKey {
	fn account() -> AccountId {
		SIDECHAIN_SIGNING_KEY.into()
	}
}

impl Network {
	pub fn process(number_of_blocks: u8) {
		for _ in 0..number_of_blocks {
			// Produce a block at the node
			AjunaNode::move_forward();
			// Produce a sidechain block
			SideChain::<SideChainSigningKey>::move_forward();
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		constants::{PLAYER_1, PLAYER_2, SUDO},
		sidechain::SideChain,
		AjunaNode, Network, Player, RuntimeBuilding, SideChainSigningKey,
		SIDECHAIN_SIGNING_KEY,
	};
	use ajuna_solo_runtime::{GameRegistry};

	#[test]
	fn play_a_guessing_game() {
		AjunaNode::default()
			.account(SUDO.into())
			.players(vec![PLAYER_1.into(), PLAYER_2.into()])
			.sidechain(SIDECHAIN_SIGNING_KEY.into())
			.build()
			.execute_with(|| {
				SideChain::<SideChainSigningKey>::build().execute_with(|| {
					// Queue players
					let player_1 = Player { account_id: PLAYER_1.into() };
					let player_2 = Player { account_id: PLAYER_2.into() };
					player_1.queue();
					assert!(GameRegistry::queued().is_none());
					player_2.queue();
					assert!(GameRegistry::queued().is_some());
					// We want to move forward by one block so the sidechain imports
					Network::process(1);
					// Game would be acknowledged by sidechain
					assert!(GameRegistry::queued().is_none());
					// Game should be created now and we can play
					player_1.play_turn(100);
					player_2.play_turn(101);
				});
			});
	}
}
