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

mod accept;
mod cancel;
mod claim;
mod create;
mod remove;
mod set_contract_collection_id;
mod set_creator;
mod set_global_config;
mod snipe;

use crate::{tests::mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::testing::H256;
