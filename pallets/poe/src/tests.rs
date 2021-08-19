use crate::{mock::*, Error, *};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_runtime::traits::BadOrigin;
use sp_std::convert::TryInto;

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		let bounded_proof: BoundedVec<u8, <Test as Config>::ClaimLimit> =
			claim.clone().try_into().unwrap();

		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		assert_eq!(
			Proofs::<Test>::get(bounded_proof),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn create_claim_user_not_signed_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		assert_noop!(PoeModule::create_claim(Origin::none(), claim.clone()), BadOrigin);
	});
}

#[test]
fn create_claim_too_long_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0; 129];
		assert_noop!(PoeModule::create_claim(Origin::signed(1), claim.clone()), Error::<Test>::ClaimLengthExceedsLimit);
	});
}

#[test]
fn create_claim_already_created_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyClaimed
		);
	});
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		let bounded_proof: BoundedVec<u8, <Test as Config>::ClaimLimit> =
			claim.clone().try_into().unwrap();

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_eq!(
			Proofs::<Test>::get(&bounded_proof),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim));
		assert_eq!(Proofs::<Test>::get(&bounded_proof), None);
	});
}

#[test]
fn revoke_claim_user_not_signed_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(PoeModule::revoke_claim(Origin::none(), claim.clone()), BadOrigin);
	});
}

#[test]
fn revoke_claim_not_created_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::NoSuchProof
		);
	});
}

#[test]
fn revoke_claim_not_owner_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotProofOwner
		);
	});
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		let bounded_proof: BoundedVec<u8, <Test as Config>::ClaimLimit> =
			claim.clone().try_into().unwrap();

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), 2, claim.clone()));
		assert_eq!(
			Proofs::<Test>::get(&bounded_proof),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn transfer_claim_user_not_signed_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		let bounded_proof: BoundedVec<u8, <Test as Config>::ClaimLimit> =
			claim.clone().try_into().unwrap();

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_eq!(
			Proofs::<Test>::get(&bounded_proof),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
		assert_noop!(PoeModule::transfer_claim(Origin::none(), 2, claim.clone()), BadOrigin);
	});
}

#[test]
fn transfer_claim_transfer_to_self_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		let bounded_proof: BoundedVec<u8, <Test as Config>::ClaimLimit> =
			claim.clone().try_into().unwrap();

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_eq!(
			Proofs::<Test>::get(&bounded_proof),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), 1, claim.clone()),
			Error::<Test>::TransferToSelf
		);
	});
}

#[test]
fn transfer_claim_not_created_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), 2, claim.clone()),
			Error::<Test>::NoSuchProof
		);
	});
}

#[test]
fn transfer_claim_not_owner_fails() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), 3, claim.clone()),
			Error::<Test>::NotProofOwner
		);
	});
}
