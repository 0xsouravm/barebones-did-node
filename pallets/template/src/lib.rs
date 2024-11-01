//! # DID Pallet
//!
//! A pallet with minimal functionality to help developers understand the essential components of
//! writing a FRAME pallet. It is typically used in beginner tutorials or in Substrate template
//! nodes as a starting point for creating a new pallet and **not meant to be used in production**.
//!
//! ## Overview
//!
//! This template pallet contains basic examples of:
//! - declaring a storage item that stores a single `u32` value
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to set a new value to storage and emits an event
//!   upon success
//! - another dispatchable function that causes a custom error to be thrown
//!
//! Each pallet section is annotated with an attribute using the `#[pallet::...]` procedural macro.
//! This macro generates the necessary code for a pallet to be aggregated into a FRAME runtime.
//!
//! Learn more about FRAME macros [here](https://docs.substrate.io/reference/frame-macros/).
//!
//! ### Pallet Sections
//!
//! The pallet sections in this template are:
//!
//! - A **configuration trait** that defines the types and parameters which the pallet depends on
//!   (denoted by the `#[pallet::config]` attribute). See: [`Config`].
//! - A **means to store pallet-specific data** (denoted by the `#[pallet::storage]` attribute).
//!   See: [`storage_types`].
//! - A **declaration of the events** this pallet emits (denoted by the `#[pallet::event]`
//!   attribute). See: [`Event`].
//! - A **declaration of the errors** that this pallet can throw (denoted by the `#[pallet::error]`
//!   attribute). See: [`Error`].
//! - A **set of dispatchable functions** that define the pallet's functionality (denoted by the
//!   `#[pallet::call]` attribute). See: [`dispatchables`].
//!
//! Run `cargo doc --package pallet-template --open` to view this pallet's documentation.

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
// #[cfg(test)]
// mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
// #[cfg(test)]
// mod tests;

mod types;
pub use types::*;

mod traits;
pub use traits::*;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
pub mod weights;
pub use weights::*;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
    // Import various useful types required by all FRAME pallets.
    use super::*;
    use crate::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
    // (`Call`s) in this pallet.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The pallet's configuration trait.
    ///
    /// All our types and constants a pallet depends on must be declared here.
    /// These types are defined generically and made concrete when the pallet is declared in the
    /// `runtime/src/lib.rs` file of your chain.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;
    }

    /// A storage item for this pallet.
    ///
    /// In this template, we are declaring a storage item called `Something` that stores a single
    /// `u32` value. Learn more about runtime storage here: <https://docs.substrate.io/build/runtime-storage/>

    #[pallet::storage]
    pub type Dids<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = DID, Value = (DidDocument, BlockNumberFor<T>)>;

    #[pallet::storage]
    pub type DidLookup<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = DID, Value = T::AccountId>;

    #[pallet::storage]
    pub type DidReverseLookup<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = T::AccountId, Value = DID>;

    /// Events that functions in this pallet can emit.
    ///
    ///
    /// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
    /// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
    /// documentation for each event field and its parameters is added to a node's metadata so it
    /// can be used by external interfaces or tools.
    ///
    ///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
    /// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
    /// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user has successfully created their DID.
        DidCreated {
            /// The account whose DID this is.
            who: T::AccountId,
            /// The DID that was created.
            did: DID,
        },

        /// A user has successfully deleted their DID.
        DidDeleted {
            /// The account whose DID this is.
            who: T::AccountId,
            /// The DID that was deleted.
            did: DID,
        },

        /// A user has successfully deleted their DID.
        DidRenewed {
            /// The account whose DID this is.
            who: T::AccountId,
            /// The DID that was renewed.
            did: DID,
        },
    }

    /// Errors that can be returned by this pallet.
    ///
    /// Errors tell users that something went wrong so it's important that their naming is
    /// informative. Similar to events, error documentation is added to a node's metadata so it's
    /// equally important that they have helpful documentation associated with them.
    ///
    /// This type of runtime error can be up to 4 bytes in size should you want to return additional
    /// information.
    #[pallet::error]
    pub enum Error<T> {
        /// The DID already exists.
        DidAlreadyExists,
        /// The User already has a DID.
        UserHasDidAlready,
        /// The DID does not exist.
        DidDoesNotExist,
        /// The DID format is invalid.
        DidFormatInvalid,
        /// The DID is expired.
        DidExpired,
        /// The DID does not need renewal
        DidDoesNotNeedRenewal,
        /// The DID is not of the caller's.
        NotOwnedDid,
    }

    /// The pallet's dispatchable functions ([`Call`]s).
    ///
    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// They must always return a `DispatchResult` and be annotated with a weight and call index.
    ///
    /// The [`call_index`] macro is used to explicitly
    /// define an index for calls in the [`Call`] enum. This is useful for pallets that may
    /// introduce new dispatchables over time. If the order of a dispatchable changes, its index
    /// will also change which will break backwards compatibility.
    ///
    /// The [`weight`] macro is used to assign a weight to each call.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a single u32 value as a parameter, writes the value
        /// to storage and emits an event.
        ///
        /// It checks that the _origin_ for this call is _Signed_ and returns a dispatch
        /// error if it isn't. Learn more about origins here: <https://docs.substrate.io/build/origins/>
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_did())]
        pub fn create_did(origin: OriginFor<T>, did: DID, metadata: DidMetadata) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            // Check if DID doesn't already exist
            ensure!(
                !Self::check_did_existence(did, who.clone()),
                Error::<T>::DidAlreadyExists
            );

            // Validate DID format.
            ensure!(Self::is_did_valid(did), Error::<T>::DidFormatInvalid);

            // Add to storages.
            // Will need to take public key as input and store it in the did document.
            // But for simplicity we are using default value for public key.
            Self::add_did_to_storages(did, metadata, who.clone());

            // Emit the DID Created event.
            Self::deposit_event(Event::DidCreated { who, did });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::create_did())]
        pub fn delete_did(origin: OriginFor<T>, did: DID) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            // Check if the DID exists
            ensure!(
                Self::check_did_existence(did, who.clone()),
                Error::<T>::DidDoesNotExist
            );

            let did_from_storage = DidReverseLookup::<T>::get(&who).unwrap();
            ensure!(did_from_storage == did, Error::<T>::NotOwnedDid); // Redundant check, but throws a better error >_<

            // Remove from storages.
            Self::remove_did_from_storages(did, who.clone());

            // Emit an event.
            Self::deposit_event(Event::DidDeleted { who, did });

            // Return a successful `DispatchResult`
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::create_did())]
        pub fn renew_did(origin: OriginFor<T>, did: DID) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            // Check if DID exists
            ensure!(
                Self::check_did_existence(did, who.clone()),
                Error::<T>::DidDoesNotExist
            );

            // Get the blocknumber at which the DID was created/renewed
            let (_did_document, block_number) = Dids::<T>::get(did).unwrap();

            // Check if the DID needs renewal
            ensure!(
                frame_system::Pallet::<T>::block_number() - block_number
                    >= BlockNumberFor::<T>::from(10u32),
                Error::<T>::DidDoesNotNeedRenewal
            );

            // Renew DID by updating the blocknumber in the storage
            Dids::<T>::mutate(did, |value| {
                if let Some((_, block_num)) = value {
                    *block_num = frame_system::Pallet::<T>::block_number();
                }
            });

            // Emit an event.
            Self::deposit_event(Event::DidRenewed { who, did });

            // Return a successful `DispatchResult`
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_did_valid(did: DID) -> bool {
            // Validate DID format.
            // DID should be 5 characters long and should not have any 
			// special characters except for alphabets and numbers and :
            // DID should be in the format did:x
            let did_length_check = did.len() == 5;
            let did_prefix_check = did.starts_with(b"did:");

            let suffix = did[4];
            let did_suffix_check = suffix.is_ascii_lowercase()
                || suffix.is_ascii_uppercase()
                || suffix.is_ascii_digit();

            did_length_check && did_prefix_check && did_suffix_check
        }

        fn check_did_existence(did: DID, who: T::AccountId) -> bool {
            DidLookup::<T>::contains_key(did)
                && Dids::<T>::contains_key(did)
                && DidReverseLookup::<T>::contains_key(who)
        }

        fn add_did_to_storages(did: DID, metadata: DidMetadata, who: T::AccountId) {
            // Add to Lookup storage.
            DidLookup::<T>::insert(did, who.clone());

            // Add to DIDs storage.
            let did_document = DidDocument {
                id: did,
                public_key: Default::default(),
                metadata,
            };
            Dids::<T>::insert(
                did,
                (did_document, frame_system::Pallet::<T>::block_number()),
            );

            // Add to Reverse Lookup storage.
            DidReverseLookup::<T>::insert(who.clone(), did);
        }

        fn remove_did_from_storages(did: DID, who: T::AccountId) {
            // Delete from Lookup storage.
            DidLookup::<T>::remove(did);

            // Delete from DIDs storage.
            Dids::<T>::remove(did);

            // Delete from Reverse Lookup storage.
            DidReverseLookup::<T>::remove(&who);
        }

        pub fn get_accountid_from_pubkey(pk: &PublicKey) -> Result<T::AccountId, codec::Error> {
            // convert a publickey to an accountId
            // TODO : Need a better way to handle the option failing?
            T::AccountId::decode(&mut &pk[..])
        }
    }
}
