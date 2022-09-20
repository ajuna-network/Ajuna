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

#[derive(Encode, Decode, Clone, PartialEq)]
pub enum GameConfigType {
	Activated = 0,
	MaxMogwaisInAccount = 1,
	MaxStashSize = 2,
	AccountNaming = 3,
}

impl Default for GameConfigType { fn default() -> Self { Self::Activated } }

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct GameConfig{
	pub parameters: [u8; GameConfig::PARAM_COUNT],
}

impl GameConfig {

	pub const PARAM_COUNT: usize = 10;

	pub fn new() -> Self {
		let parameters = [0; GameConfig::PARAM_COUNT];

		return GameConfig {
			parameters: parameters,
		};
	}
	pub fn config_value(index: u8, value: u8) -> u32 {
		let result:u32;
		match index {
			// MaxMogwaisInAccount
            1 => {
				match value {
					0 => result = 6,
					1 => result = 12,
					2 => result = 18,
					3 => result = 24,
					_ => result = 0,
				}
			},
            _ => result = 0,
		}
		result
	}
	pub fn verify_update(index: u8, value: u8, update_value_opt: Option<u8>) -> u8 {
		let mut result:u8;
		match index {
			// MaxMogwaisInAccount
            1 => {
				match value {
					0 => result = 1,
					1 => result = 2,
					2 => result = 3,
					_ => result = 0,
				}
			},
            _ => result = 0,
		}
		// don't allow bad requests
		if update_value_opt.is_some() && result != update_value_opt.unwrap() {
			result = 0;
		}
		result
	}
}