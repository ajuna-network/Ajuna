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

#[cfg(feature = "solo")]
use ajuna_service::{ajuna_solo_runtime as runtime, solo::FullClient};

use sc_cli::Result;
use sc_client_api::BlockBackend;
use sp_core::{Encode, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{OpaqueExtrinsic, SaturatedConversion};

use std::{sync::Arc, time::Duration};

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct BenchmarkExtrinsicBuilder {
	client: Arc<FullClient>,
}

impl BenchmarkExtrinsicBuilder {
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Arc<FullClient>) -> Self {
		Self { client }
	}
}

impl frame_benchmarking_cli::ExtrinsicBuilder for BenchmarkExtrinsicBuilder {
	fn remark(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			self.client.as_ref(),
			acc,
			frame_system::Call::remark { remark: vec![] }.into(),
			nonce,
		)
		.into();

		Ok(extrinsic)
	}
}

/// Create a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
pub fn create_benchmark_extrinsic(
	client: &FullClient,
	sender: sp_core::sr25519::Pair,
	call: runtime::Call,
	nonce: u32,
) -> runtime::UncheckedExtrinsic {
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let best_hash = client.chain_info().best_hash;
	let best_block = client.chain_info().best_number;

	let period = runtime::BlockHashCount::get()
		.checked_next_power_of_two()
		.map(|c| c / 2)
		.unwrap_or(2) as u64;
	let extra: runtime::SignedExtra = (
		// TODO: Integrate upstream-changes after scs/substrate-api-client#211 has been solved.
		// frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
		frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
		frame_system::CheckTxVersion::<runtime::Runtime>::new(),
		frame_system::CheckGenesis::<runtime::Runtime>::new(),
		frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
			period,
			best_block.saturated_into(),
		)),
		frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
		frame_system::CheckWeight::<runtime::Runtime>::new(),
		// TODO PLAT-276: reinstate ChargeTransactionPayment once worker supports it
		pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
	);

	let raw_payload = runtime::SignedPayload::from_raw(
		call.clone(),
		extra.clone(),
		(
			// TODO: Integrate upstream-changes after scs/substrate-api-client#211 has been solved.
			// (),
			runtime::VERSION.spec_version,
			runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| sender.sign(e));

	runtime::UncheckedExtrinsic::new_signed(
		call,
		sp_runtime::AccountId32::from(sender.public()).into(),
		ajuna_primitives::Signature::Sr25519(signature),
		extra,
	)
}

/// Generates inherent data for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub fn inherent_benchmark_data() -> Result<InherentData> {
	let mut inherent_data = InherentData::new();
	let d = Duration::from_millis(0);
	let timestamp = sp_timestamp::InherentDataProvider::new(d.into());

	timestamp
		.provide_inherent_data(&mut inherent_data)
		.map_err(|e| format!("creating inherent data: {:?}", e))?;
	Ok(inherent_data)
}
