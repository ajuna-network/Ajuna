pub mod constants {
    use frame_support::{
        parameter_types,
        weights::{constants, Weight},
    };

    parameter_types! {
		/// Executing a NO-OP `System::remarks` Extrinsic.
		pub const ExtrinsicBaseWeight: Weight = 125_000 * constants::WEIGHT_PER_NANOS;
	}

    #[cfg(test)]
    mod test_weights {
        use frame_support::weights::constants;

        /// Checks that the weight exists and is sane.
        // NOTE: If this test fails but you are sure that the generated values are fine,
        // you can delete it.
        #[test]
        fn sane() {
            let w = super::constants::ExtrinsicBaseWeight::get();

            // At least 10 µs.
            assert!(w >= 10 * constants::WEIGHT_PER_MICROS, "Weight should be at least 10 µs.");
            // At most 1 ms.
            assert!(w <= constants::WEIGHT_PER_MILLIS, "Weight should be at most 1 ms.");
        }
    }
}
