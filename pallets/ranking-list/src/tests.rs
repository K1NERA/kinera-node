use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		
	});
}



// create_ranking_list


// add_internal_movie_to_ranking_list


// add_external_movie_to_ranking_list


// vote_for


// claim_ranking_rewards





// create_list_deadline


// do_resolve_festivals_deadline


// resolve_ranking_list


// do_calculate_voting_power


// do_resolve_festivals_deadline

