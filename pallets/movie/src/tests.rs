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
		// Ensure the expected error is thrown when no value is present.
		
	});
}


// create_internal_movie

// create_external_movie




// do_create_internal_movie

// do_create_external_movie

// do_ensure_internal_movie_exist

// do_does_external_movie_exist

// do_ensure_external_movie_doesnt_exist

// do_ensure_external_movie_exists

// get_movie_uploader