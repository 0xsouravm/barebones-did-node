//! # DID(Decentralized Identifier) Pallet
//!
//! The DID Pallet provides basic functionality for managing Decentralized Identifiers (DIDs).
//! It serves as an example to help developers understand the essential components of writing
//! a FRAME pallet, while also introducing them to DIDs and some important functionalites that
//! you would want to see in a DID pallet. It is intended to used for beginner tutorials as a
//! starting point for creating a new pallet that deals with DIDs and **not meant to be used
//! in production**.
//!
//! ## Overview
//!
//! This did pallet contains examples of:
//! - declaring a storage item that maps a `DID` to a tuple of `(DidDocument, BlockNumber)`
//! - declaring a storage item that maps a `DID` to an `AccountId`
//! - declaring a storage item that maps a `AccountId` to a `DID`
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to create a DID.
//! - a dispatchable function that allows a user to delete a DID.
//! - a dispatchable function that allows a user to renew a DID.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod types;
pub use types::*;

// mod traits;
// pub use traits::*;

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
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_did())]
        /// Creates a new Decentralized Identifier (DID).
        ///
        /// This function allows a user to create a new DID with associated metadata. It performs several checks:
        /// - Ensures the extrinsic was signed.
        /// - Checks if the DID already exists.
        /// - Validates the format of the DID.
        ///
        /// If all checks pass, the DID is added to storage and an event is emitted.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the extrinsic, which must be signed.
        /// * `did` - The Decentralized Identifier to be created.
        /// * `metadata` - Metadata associated with the DID.
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - The DID already exists.
        /// - The DID format is invalid.
        ///
        /// # Events
        ///
        /// Emits a `DidCreated` event upon successful creation of the DID.
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
        /// Deletes an existing Decentralized Identifier (DID).
        ///
        /// This function allows a user to delete an existing DID. It performs several checks:
        /// - Ensures the extrinsic was signed.
        /// - Checks if the DID exists.
        /// - Ensures the DID belongs to the signer.
        ///
        /// If all checks pass, the DID is removed from storage and an event is emitted.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the extrinsic, which must be signed.
        /// * `did` - The Decentralized Identifier to be deleted.
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - The DID does not exist.
        /// - The DID does not belong to the signer.
        ///
        /// # Events
        ///
        /// Emits a `DidDeleted` event upon successful deletion of the DID.
        pub fn delete_did(origin: OriginFor<T>, did: DID) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            let did_from_storage = DidReverseLookup::<T>::get(&who).unwrap();
            ensure!(did_from_storage == did, Error::<T>::NotOwnedDid); // Redundant check, but throws a better error >_<

            // Check if the DID exists
            ensure!(
                Self::check_did_existence(did, who.clone()),
                Error::<T>::DidDoesNotExist
            );

            // Remove from storages.
            Self::remove_did_from_storages(did, who.clone());

            // Emit an event.
            Self::deposit_event(Event::DidDeleted { who, did });

            // Return a successful `DispatchResult`
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::create_did())]
        /// Renews an existing Decentralized Identifier (DID).
        ///
        /// This function allows a user to renew an existing DID. It performs several checks:
        /// - Ensures the extrinsic was signed.
        /// - Checks if the DID exists.
        /// - Checks if the DID needs renewal based on the block number.
        ///
        /// If all checks pass, the block number associated with the DID is updated in storage and an event is emitted.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the extrinsic, which must be signed.
        /// * `did` - The Decentralized Identifier to be renewed.
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - The DID does not exist.
        /// - The DID does not need renewal.
        ///
        /// # Events
        ///
        /// Emits a `DidRenewed` event upon successful renewal of the DID.
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
        /// Validates the format of a Decentralized Identifier (DID).
        ///
        /// This function checks if the provided DID adheres to the expected format:
        /// - The DID should be exactly 5 characters long.
        /// - The DID should start with the prefix "did:".
        /// - The last character of the DID should be an alphabet (uppercase or lowercase) or a digit.
        ///
        /// # Arguments
        ///
        /// * `did` - The Decentralized Identifier to be validated.
        ///
        /// # Returns
        ///
        /// Returns `true` if the DID is valid, otherwise returns `false`.
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

        /// Checks if a Decentralized Identifier (DID) exists.
        ///
        /// This function checks if the provided DID exists in the storage by verifying its presence
        /// in the `DidLookup`, `Dids`, and `DidReverseLookup` storages.
        ///
        /// # Arguments
        ///
        /// * `did` - The Decentralized Identifier to be checked.
        /// * `who` - The account ID associated with the DID.
        ///
        /// # Returns
        ///
        /// Returns `true` if the DID exists, otherwise returns `false`.
        fn check_did_existence(did: DID, who: T::AccountId) -> bool {
            DidLookup::<T>::contains_key(did)
                && Dids::<T>::contains_key(did)
                && DidReverseLookup::<T>::contains_key(who)
        }

        /// Adds a Decentralized Identifier (DID) to the storages.
        ///
        /// This function adds the provided DID and its associated metadata to the `DidLookup`, `Dids`,
        /// and `DidReverseLookup` storages.
        ///
        /// # Arguments
        ///
        /// * `did` - The Decentralized Identifier to be added.
        /// * `metadata` - Metadata associated with the DID.
        /// * `who` - The account ID associated with the DID.
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

        /// Removes a Decentralized Identifier (DID) from the storages.
        ///
        /// This function removes the provided DID and its associated data from the `DidLookup`, `Dids`,
        /// and `DidReverseLookup` storages.
        ///
        /// # Arguments
        ///
        /// * `did` - The Decentralized Identifier to be removed.
        /// * `who` - The account ID associated with the DID.
        fn remove_did_from_storages(did: DID, who: T::AccountId) {
            // Delete from Lookup storage.
            DidLookup::<T>::remove(did);

            // Delete from DIDs storage.
            Dids::<T>::remove(did);

            // Delete from Reverse Lookup storage.
            DidReverseLookup::<T>::remove(&who);
        }
    }
}
