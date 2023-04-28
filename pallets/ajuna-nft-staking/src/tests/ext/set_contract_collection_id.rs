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

use super::*;

#[test]
fn works() {
	ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
		let collection_id = create_collection(ALICE);
		assert_eq!(ContractCollectionId::<Test>::get(), None);
		assert_ok!(NftStake::set_contract_collection_id(
			RuntimeOrigin::signed(ALICE),
			collection_id
		));
		assert_eq!(ContractCollectionId::<Test>::get(), Some(collection_id));
		System::assert_last_event(RuntimeEvent::NftStake(crate::Event::ContractCollectionSet {
			collection_id,
		}));
	});
}

#[test]
fn rejects_non_existing_collection() {
	ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
		assert_noop!(
			NftStake::set_contract_collection_id(RuntimeOrigin::signed(ALICE), 17),
			Error::<Test>::UnknownCollection
		);
	});
}

#[test]
fn rejects_non_creator_owned_collection() {
	ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
		let collection_id = create_collection(BOB);
		assert_noop!(
			NftStake::set_contract_collection_id(RuntimeOrigin::signed(ALICE), collection_id),
			Error::<Test>::Ownership
		);
	});
}
