pub mod constants {
	use frame_support::{
		parameter_types,
		weights::{constants, Weight},
	};

	parameter_types! {
		/// Importing a block with 0 Extrinsics.
		pub const BlockExecutionWeight: Weight = 5_000_000 * constants::WEIGHT_PER_NANOS;
	}

	#[cfg(test)]
	mod test_weights {
		use frame_support::weights::constants;

		/// Checks that the weight exists and is sane.
		// NOTE: If this test fails but you are sure that the generated values are fine,
		// you can delete it.
		#[test]
		fn sane() {
			let w = super::constants::BlockExecutionWeight::get();

			// At least 100 µs.
			assert!(w >= 100 * constants::WEIGHT_PER_MICROS, "Weight should be at least 100 µs.");
			// At most 50 ms.
			assert!(w <= 50 * constants::WEIGHT_PER_MILLIS, "Weight should be at most 50 ms.");
		}
	}
}
