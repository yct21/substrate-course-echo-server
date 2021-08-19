#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::convert::TryInto;
	use sp_std::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The maximum length of a claim
		#[pallet::constant]
		type ClaimLimit: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn get_proofs)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Proofs<T: Config> =
		StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::ClaimLimit>, (T::AccountId, T::BlockNumber), OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		ClaimCreated(T::AccountId, BoundedVec<u8, T::ClaimLimit>),
		ClaimRevoked(T::AccountId, BoundedVec<u8, T::ClaimLimit>),
		ClaimTransferred(T::AccountId, T::AccountId, BoundedVec<u8, T::ClaimLimit>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyClaimed,
		NoSuchProof,
		NotProofOwner,
		TransferToSelf,
		ClaimLengthExceedsLimit,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn create_claim(origin: OriginFor<T>, proof: Vec<u8>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let bounded_proof: BoundedVec<u8, T::ClaimLimit> =
				proof.try_into().map_err(|_| Error::<T>::ClaimLengthExceedsLimit)?;

			ensure!(!Proofs::<T>::contains_key(&bounded_proof), Error::<T>::ProofAlreadyClaimed);
			let current_block = <frame_system::Pallet<T>>::block_number();
			Proofs::<T>::insert(&bounded_proof, (&sender, current_block));
			Self::deposit_event(Event::ClaimCreated(sender, bounded_proof));

			Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn revoke_claim(origin: OriginFor<T>, proof: Vec<u8>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let bounded_proof: BoundedVec<u8, T::ClaimLimit> =
				proof.try_into().map_err(|_| Error::<T>::ClaimLengthExceedsLimit)?;

			let (owner, _block_number) = Proofs::<T>::get(&bounded_proof).ok_or(Error::<T>::NoSuchProof)?;
			ensure!(sender == owner, Error::<T>::NotProofOwner);
			Proofs::<T>::remove(&bounded_proof);

			Self::deposit_event(Event::ClaimRevoked(sender, bounded_proof));
			Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn transfer_claim(
			from: OriginFor<T>,
			to: T::AccountId,
			proof: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(from)?;
			ensure!(sender != to, Error::<T>::TransferToSelf);
			let bounded_proof: BoundedVec<u8, T::ClaimLimit> =
				proof.try_into().map_err(|_| Error::<T>::ClaimLengthExceedsLimit)?;
			let (owner, _block_number) = Proofs::<T>::get(&bounded_proof).ok_or(Error::<T>::NoSuchProof)?;

			ensure!(sender == owner, Error::<T>::NotProofOwner);

			let current_block = <frame_system::Pallet<T>>::block_number();
			Proofs::<T>::insert(&bounded_proof, (&to, current_block));

			Self::deposit_event(Event::ClaimTransferred(sender, to, bounded_proof));
			Ok(().into())
		}
	}
}
