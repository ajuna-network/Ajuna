use frame_system::Config;
use sp_io::TestExternalities;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, One},
	Storage,
};

// Some useful accounts
pub const SIDECHAIN_SIGNING_KEY: [u8; 32] = [0x1; 32];
pub const SUDO: [u8; 32] = [0x2; 32];
pub const PLAYER_1: [u8; 32] = [0x3; 32];
pub const PLAYER_2: [u8; 32] = [0x4; 32];

pub trait BlockNumbers<B> {
	fn block_number() -> B;
	fn set_block_number(n: B);
	fn current_block_number() -> B;
}

pub trait RuntimeBuilding<Runtime: Config, B: AtLeast32BitUnsigned + One, T: BlockNumbers<B>> {
	fn configure_storages(&self, _storage: &mut Storage);

	fn build(&self) -> TestExternalities {
		let mut storage =
			frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		self.configure_storages(&mut storage);

		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| T::set_block_number(One::one()));

		ext
	}
}

pub trait BlockProcessing<B: Copy + AtLeast32BitUnsigned + One, T: BlockNumbers<B>> {
	fn move_forward() {
		Self::move_forward_blocks(One::one());
	}

	fn move_forward_blocks(n: B) {
		let next_block_number = T::current_block_number().saturating_add(n);
		while T::block_number() < next_block_number {
			Self::on_block();
			T::set_block_number(next_block_number);
		}
	}

	fn on_block();
}

#[macro_export]
macro_rules! impl_block_numbers {
	($system:ty, $block_number:ty) => {
		use crate::constants::{BlockNumbers};
		use sp_runtime::{traits::BlockNumberProvider};

		pub struct RuntimeBlocks;
		impl BlockNumbers<$block_number> for RuntimeBlocks {
			fn block_number() -> $block_number {
				<$system>::block_number()
			}

			fn set_block_number(n: $block_number) {
				<$system>::set_block_number(n)
			}

			fn current_block_number() -> $block_number {
				<$system>::current_block_number()
			}
		}
	};
}