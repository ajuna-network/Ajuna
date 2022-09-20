// DOT Mog, Susbstrate Gamification Project with C# .NET Standard & Unity3D
// Copyright (C) 2020-2021 DOT Mog Team, darkfriend77 & metastar77
//
// DOT Mog is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License.
// DOT Mog is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

use frame_support::{codec::{Encode, Decode}};
//use sp_runtime::{traits::{Hash}};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, PartialEq, TypeInfo)]
pub enum GameEventType {
	Default = 0,
	Hatch = 1,
}

impl Default for GameEventType { fn default() -> Self { Self::Default } }

impl GameEventType {

	pub fn time_till(game_type: GameEventType) -> u16 {
		match game_type {
			GameEventType::Hatch => 100,
			GameEventType::Default => 0,
		}
	}

	pub fn duration(game_type: GameEventType) -> u16 {
		match game_type {
			GameEventType::Hatch => 0,
			GameEventType::Default => 0,
		}
	}
}