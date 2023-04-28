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
		let new_config = GlobalConfig { pallet_locked: true };
		assert_ok!(NftStake::set_global_config(RuntimeOrigin::signed(ALICE), new_config));
		assert_eq!(GlobalConfigs::<Test>::get(), new_config);
		System::assert_last_event(RuntimeEvent::NftStake(crate::Event::SetGlobalConfig {
			new_config,
		}));
	});
}

#[test]
fn rejects_non_creator_calls() {
	ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
		assert_noop!(
			NftStake::set_global_config(RuntimeOrigin::signed(BOB), GlobalConfig::default()),
			DispatchError::BadOrigin
		);
	});
}
