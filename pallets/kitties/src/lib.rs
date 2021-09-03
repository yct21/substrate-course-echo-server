#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet]
pub mod pallet {
	use codec::{FullCodec, FullEncode};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded};

	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8; 16]);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: pallet_balances::Config + frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type CurrencyReservedForKitty: Get<BalanceOf<Self>>;
		type KittyIndex: AtLeast32BitUnsigned
			+ Bounded
			+ FullCodec
			+ FullEncode
			+ core::ops::Add<u32, Output = Self::KittyIndex> // to increase kitty index
			+ Clone
			+ core::fmt::Debug;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex),
		KittyBought(T::AccountId, T::AccountId, T::KittyIndex, BalanceOf<T>),
		KittySold(T::AccountId, T::AccountId, T::KittyIndex, BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// kitties amount exceeds the limit of kitty index
		KittiesCountOverflow,

		/// user is not owner of this kitty
		NotOwner,

		/// user try to transfer a kitty to self
		TransferToSelf,

		/// breed between same kitty
		SameParentIndex,

		/// kitty not exist
		InvalidKittyIndex,

		/// Not enough balance to create or trade kitty
		NotEnoughBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// create a kitty
		#[pallet::weight(1000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::kitties_count().unwrap_or(T::KittyIndex::min_value());
			ensure!(kitty_id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
			T::Currency::reserve(&who, T::CurrencyReservedForKitty::get())
				.map_err(|_| Error::<T>::NotEnoughBalance)?;

			let dna = Self::random_value(&who);

			Kitties::<T>::insert(kitty_id.clone(), Kitty(dna));
			Owner::<T>::insert(kitty_id.clone(), who.clone());
			// It won't overflow since we checked it previously
			KittiesCount::<T>::put(kitty_id.clone() + 1);

			Self::deposit_event(Event::KittyCreated(who, kitty_id.clone()));

			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn transfer(
			origin: OriginFor<T>,
			new_owner: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::transfer_ownership(who.clone(), new_owner.clone(), kitty_id.clone())?;
			Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));

			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn buy(
			buyer: OriginFor<T>,
			seller: T::AccountId,
			kitty_id: T::KittyIndex,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(buyer)?;

			Self::transfer_ownership(seller.clone(), buyer.clone(), kitty_id.clone())?;
			Self::transfer_balance(buyer.clone(), seller.clone(), price)?;
			Self::deposit_event(Event::KittyBought(buyer, seller, kitty_id, price));

			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn sell(
			seller: OriginFor<T>,
			buyer: T::AccountId,
			kitty_id: T::KittyIndex,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let seller = ensure_signed(seller)?;

			Self::transfer_ownership(seller.clone(), buyer.clone(), kitty_id.clone())?;
			Self::transfer_balance(buyer.clone(), seller.clone(), price)?;
			Self::deposit_event(Event::KittySold(seller, buyer, kitty_id, price));

			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_id = Self::kitties_count().unwrap_or(T::KittyIndex::min_value());
			ensure!(kitty_id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);

			// mix parents' dna
			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;
			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];
			for i in 0..dna_1.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
			}

			Kitties::<T>::insert(kitty_id.clone(), Kitty(new_dna));
			Owner::<T>::insert(kitty_id.clone(), who.clone());
			KittiesCount::<T>::put(kitty_id.clone() + 1);

			Self::deposit_event(Event::KittyCreated(who, kitty_id));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128)
		}

		fn transfer_ownership(
			from: T::AccountId,
			to: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> Result<(), Error<T>> {
			let owner = Owner::<T>::get(&kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
			ensure!(from == owner, Error::<T>::NotOwner);
			ensure!(from != to, Error::<T>::TransferToSelf);

			Owner::<T>::insert(kitty_id.clone(), to.clone());

			Ok(())
		}

		fn transfer_balance(
			from: T::AccountId,
			to: T::AccountId,
			price: BalanceOf<T>,
		) -> Result<(), DispatchError> {
			T::Currency::reserve(&to, T::CurrencyReservedForKitty::get() + price)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;
			T::Currency::unreserve(&from, T::CurrencyReservedForKitty::get() + price);
			T::Currency::unreserve(&to, price);
			T::Currency::transfer(&from, &to, price, ExistenceRequirement::KeepAlive)
		}
	}
}
