use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_dotmog_breeding() {
	new_test_ext().execute_with(|| {
		
		assert_eq!(BattleMogs::all_mogwais_count(), 0);
		
		assert_ok!(BattleMogs::create_mogwai(Origin::signed(1)));
		assert_eq!(BattleMogs::all_mogwais_count(), 1);
		
		// test create
		assert_ok!(BattleMogs::create_mogwai(Origin::signed(1)));
		assert_eq!(BattleMogs::all_mogwais_count(), 2);

		let mogwai_hash_1 = BattleMogs::mogwai_by_index(0);
		let mogwai_hash_2 = BattleMogs::mogwai_by_index(1);
		let mogwai_1 = BattleMogs::mogwai(mogwai_hash_1);
		let mogwai_2 = BattleMogs::mogwai(mogwai_hash_2);

		assert_ne!(mogwai_1.gen, 0);
		assert_ne!(mogwai_2.gen, 0);

		assert_eq!(System::block_number(), 0);
		run_to_block(101);
		assert_eq!(System::block_number(), 101);

		// test morph
		assert_ok!(BattleMogs::hatch_mogwai(Origin::signed(1), mogwai_hash_1));
		assert_ok!(BattleMogs::hatch_mogwai(Origin::signed(1), mogwai_hash_2));

		// test morph
		//assert_ok!(BattleMogs::morph_mogwai(Origin::signed(1), mogwai_hash_1));
		
		// test breed
		//assert_ok!(BattleMogs::breed_mogwai(Origin::signed(1), mogwai_hash_1, mogwai_hash_2));
		//assert_eq!(BattleMogs::all_mogwais_count(), 3);

		// create real mogwai by breeding
		//let mogwai_hash_3 = BattleMogs::mogwai_by_index(2);
		//let mogwai_3 = BattleMogs::mogwai(mogwai_hash_3);
		//assert_eq!(mogwai_3.gen, 1);

		// run forward 100 blocks to make the egg hatch
		//assert_eq!(System::block_number(), 0);
		//run_to_block(101);
		//assert_eq!(System::block_number(), 101);

	});
}