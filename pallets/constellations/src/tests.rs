use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		// assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		// assert_eq!(TemplateModule::something(), 97);
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
             
	});
}


// create_festival

// activate_festival

// activate_festival_asap

// add_movies_to_fest

// remove_movies_from_fest

// vote_for_movie_in_festival

// claim_festival_rewards




// do_create_festival

// do_bind_owners_to_festival

// do_bind_duration_to_festival

// do_create_empty_block_assignments

// hook_activate_festival

// hook_deactivate_festival

// do_active_to_finished_fest_ownership

// do_vote_for_movie_in_festival

// account_id

// do_resolve_market

// do_get_winning_options

// do_get_winners_total_lockup

// do_calculate_simple_reward