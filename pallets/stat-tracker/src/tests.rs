use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		// assert_ok!(StatTrackerModule::do_something(Origin::signed(1), 42));
		// // Read pallet storage and assert an expected result.
		// assert_eq!(StatTrackerModule::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		// assert_noop!(StatTrackerModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}


//** Extrinsic Tests **//

	
	// Should pass.
	#[test]
	fn register_new_wallet() {
		new_test_ext().execute_with(|| {
			// Register a wallet with an unregistered address. Should pass.

			// Register a wallet with an already registered address. Should fail.

			// Register a wallet with an invalid address. Should fail.
			
		});
	}





// register_new_wallet

// unregister_wallet

// update_wallet_data

// claim_all_tokens



// do_update_wallet_tokens

// do_update_wallet_tokens_doesnt_exist

// do_update_wallet_tokens_exists

// do_update_wallet_imbalance

// do_handle_imbalance_wallet_doesnt_exist

// do_handle_imbalance_wallet_exists

// do_calculate_token_change

// do_calculate_imbalance_change

// do_calculate_reputation_change

// do_is_wallet_registered

// account_id
