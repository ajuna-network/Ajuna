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

pub mod v1;
pub mod v2;
pub mod v3;

use super::*;
use frame_support::traits::OnRuntimeUpgrade;

// The current storage version.
pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

const LOG_TARGET: &str = "runtime::ajuna-awesome-avatars";
