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
//! - declaring a storage item that maps a `DID` and `AccountId` to an empty tuple
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to create a DID.
//! - a dispatchable function that allows a user to delete a DID.
//! - a dispatchable function that allows a user to delegate a DID.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::{fungible::MutateHold, GetCallMetadata};
pub use pallet::*;

mod types;
pub use types::*;
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
    use frame_support::{
        pallet_prelude::*,
        traits::{fungible, tokens::Precision},
    };

    use frame_system::pallet_prelude::*;

    pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    pub type CallOf<T> = <T as frame_system::Config>::RuntimeCall;

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
        /// Overarching hold reason. Our `HoldReason` below will become a part of this "Outer Enum"
        /// thanks to the `#[runtime]` macro.
        type RuntimeHoldReason: From<HoldReason>;
        /// Type to access the Balances Pallet.
        type NativeBalance: fungible::Inspect<Self::AccountId>
            + fungible::Mutate<Self::AccountId>
            + fungible::hold::Inspect<Self::AccountId>
            + fungible::hold::Mutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
            + fungible::freeze::Inspect<Self::AccountId>
            + fungible::freeze::Mutate<Self::AccountId>;
        /// Type for the amount to reserve for creating a DID.
        type HoldAmount: Get<BalanceOf<Self>>;
        /// A type representing the validity duration of a temporary DID.
        type TempDidValidity: Get<BlockNumberFor<Self>>;
    }

    // Storage items for this pallet.
    /// Storage map for storing Decentralized Identifiers (DIDs) along with the
    /// DID document and block number at which it was created.
    #[pallet::storage]
    pub type Dids<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = DID, Value = (DidDocument, BlockNumberFor<T>)>;

    /// Storage map for looking up the account ID associated with a given DID.
    #[pallet::storage]
    pub type DidLookup<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = DID, Value = T::AccountId>;

    /// Storage map for reverse lookup of DIDs based on the user's account ID.
    #[pallet::storage]
    pub type DidReverseLookup<T: Config> =
        StorageMap<Hasher = Blake2_128Concat, Key = T::AccountId, Value = DID>;

    /// Storage double map to map DID and Delegate account ID with an empty tuple.
    #[pallet::storage]
    pub type Delegations<T: Config> = StorageDoubleMap<
        Hasher1 = Blake2_128Concat,
        Hasher2 = Blake2_128Concat,
        Key1 = T::AccountId,
        Key2 = DID,
        Value = (),
    >;

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
        /// A user has successfully delegated their DID.
        DidDelegated {
            /// The account whose DID this is.
            who: T::AccountId,
            /// The DID that was renewed.
            did: DID,
            /// The new owner of the DID.
            delegate: T::AccountId,
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
        /// The account ID is already delegated.
        UserAlreadyDelegated,
        /// The account ID already owns a DID.
        UserAlreadyHasDid,
    }

    /// A reason for the pallet placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        /// Funds are held to create and operate DID.
        #[codec(index = 0)]
        DidOwningHold,
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
        /// This function allows a user to create a new DID with associated metadata by holding some
        /// funds of the user. It performs several checks:
        /// - Ensures the extrinsic was signed.
        /// - Checks if the DID already exists.
        /// - Validates the format of the DID.
        /// - Holds a specified amount of the caller's balance.
        ///
        /// If all checks pass, the DID is added to storage and an event is emitted.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the extrinsic, which must be signed.
        /// * `did` - The Decentralized Identifier to be created.
        /// * `metadata` - Metadata associated with the DID.
        /// * `did_type` - The type of DID to be created.
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - The DID already exists.
        /// - The DID format is invalid.
        /// - The user does not have enough balance to hold the specified amount.
        ///
        /// # Events
        ///
        /// Emits a `DidCreated` event upon successful creation of the DID.
        pub fn create_did(
            origin: OriginFor<T>,
            did: DID,
            metadata: DidMetadata,
            did_type: DidType,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            // Check if DID doesn't already exist
            ensure!(
                !Self::check_did_existence(did, who.clone()),
                Error::<T>::DidAlreadyExists
            );

            // Validate DID format.
            ensure!(
                Self::is_did_valid(did, did_type),
                Error::<T>::DidFormatInvalid
            );

            // Hold the amount only if it is a regular DID.
            if did_type == DidType::Reg {
                T::NativeBalance::hold(
                    &HoldReason::DidOwningHold.into(),
                    &who,
                    T::HoldAmount::get(),
                )?;
            }

            // Add to storages.
            // Will need to take public key as input and store it in the did document.
            // But for simplicity we are using default value for public key.
            Self::add_did_to_storages(did, metadata, who.clone(), did_type);

            // Emit the DID Created event.
            Self::deposit_event(Event::DidCreated { who, did });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::create_did())]
        /// Deletes an existing Decentralized Identifier (DID).
        ///
        /// This function allows a user to delete an existing DID and releases the held funds. It performs several checks:
        /// - Ensures the extrinsic was signed.
        /// - Checks if the DID exists.
        /// - Ensures the DID belongs to the signer.
        /// - Releases the held funds.
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

            // Release the hold.
            T::NativeBalance::release(
                &HoldReason::DidOwningHold.into(),
                &who,
                T::HoldAmount::get(),
                Precision::Exact,
            )?;

            // Emit an event.
            Self::deposit_event(Event::DidDeleted { who, did });

            // Return a successful `DispatchResult`
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::create_did())]
        /// Delegates ownership of a Decentralized Identifier (DID) to another account.
        ///
        /// This function allows the current owner of a DID to delegate ownership to another account. It performs several checks:
        /// - Ensures the extrinsic was signed.
        /// - Checks if the DID exists.
        /// - Ensures the DID belongs to the signer.
        /// - Checks if the delegate already has a DID.
        /// - Checks if the account ID is already delegated to some other DID.
        ///
        /// If all checks pass, the ownership of the DID is updated in storage and an event is emitted.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the extrinsic, which must be signed.
        /// * `did` - The Decentralized Identifier to be delegated.
        /// * `delegate` - The account ID of the new owner.
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - The DID does not exist.
        /// - The DID does not belong to the signer.
        /// - The delegate already has a DID.
        /// - The account ID is already delegated.
        ///
        /// # Events
        ///
        /// Emits a `DidDelegated` event upon successful delegation of the DID.
        pub fn delegate_ownership(
            origin: OriginFor<T>,
            did: DID,
            delegate: T::AccountId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            // Ensure the DID belongs to the signer
            let did_from_storage =
                DidReverseLookup::<T>::get(&who).ok_or(Error::<T>::DidDoesNotExist)?;
            ensure!(did_from_storage == did, Error::<T>::NotOwnedDid); // Redundant check, but throws a better error >_<

            // Check if the DID exists
            ensure!(
                Self::check_did_existence(did, who.clone()),
                Error::<T>::DidDoesNotExist
            );

            // Check if the delegation already has a DID
            ensure!(
                !DidReverseLookup::<T>::contains_key(&delegate),
                Error::<T>::UserAlreadyHasDid
            );

            // Check if the account ID is already delegated
            ensure!(
                !Delegations::<T>::contains_key(delegate.clone(), did),
                Error::<T>::UserAlreadyDelegated
            );

            // Add the delegate to the list of delegations
            Delegations::<T>::insert(delegate.clone(), did, ());

            // Emit an event.
            Self::deposit_event(Event::DidDelegated { who, did, delegate });

            // Return a successful `DispatchResult`
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::create_did())]
        // Task - 0 - Implement the remove delegation extrinsic

        /// Removes a delegation of a Decentralized Identifier (DID).
        ///
        /// This function allows the owner of a DID or the delegate to remove the delegation. It performs several checks:
        /// - Ensures the extrinsic was signed.
        /// - Checks if the DID exists.
        /// - Ensures the caller is either the owner or the delegate.
        ///
        /// If all checks pass, the delegation is removed from storage and an event is emitted.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the extrinsic, which must be signed.
        /// * `did` - The Decentralized Identifier whose delegation is to be removed.
        /// * `delegate` - The account ID of the delegate to be removed.
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - The DID does not exist.
        /// - The caller is neither the owner nor the delegate.
        ///
        /// # Events
        ///
        /// Emits a `DidDelegationRemoved` event upon successful removal of the delegation.
        pub fn remove_delgation(
            _origin: OriginFor<T>,
            _did: DID,
            _delegate: T::AccountId,
        ) -> DispatchResult {
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Validates the format of a Decentralized Identifier (DID).
        ///
        /// This function checks if the provided DID adheres to the expected format:
        /// - The DID should be exactly 5 characters long.
        /// - The DID should start with the prefix "did:ssid:" or "did:temp:".
        /// - The last character of the DID should be an alphabet (uppercase or lowercase) or a digit.
        ///
        /// # Arguments
        ///
        /// * `did` - The Decentralized Identifier to be validated.
        /// * `did_type` - The type of DID to be validated.
        ///
        /// # Returns
        ///
        /// Returns `true` if the DID is valid, otherwise returns `false`.
        fn is_did_valid(did: DID, did_type: DidType) -> bool {
            // Validate DID format.
            // DID should be 5 characters long and should not have any
            // special characters except for alphabets and numbers and :
            // DID should be in the format did:ssid:x for regular DIDs and
            // did:temp:x for temporary DIDs
            let did_prefix_check = match did_type {
                DidType::Reg => did.starts_with(b"did:ssid:"),
                DidType::Temp => did.starts_with(b"did:temp:"),
            };
            let did_length_check = did.len() == 10;

            let suffix = did[9];
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
                || Dids::<T>::contains_key(did)
                || DidReverseLookup::<T>::contains_key(who)
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
        /// * `did_type` - The type of DID to be added.
        fn add_did_to_storages(
            did: DID,
            metadata: DidMetadata,
            who: T::AccountId,
            did_type: DidType,
        ) {
            // Add to Lookup storage.
            DidLookup::<T>::insert(did, who.clone());

            let did_document = DidDocument {
                id: did,
                public_key: Default::default(),
                metadata,
                did_type,
            };

            // Add to DIDs storage.
            match did_type {
                DidType::Reg => {
                    Dids::<T>::insert(
                        did,
                        (did_document, frame_system::Pallet::<T>::block_number()),
                    );
                }
                DidType::Temp => {
                    Dids::<T>::insert(
                        did,
                        (
                            did_document,
                            frame_system::Pallet::<T>::block_number() + T::TempDidValidity::get(),
                        ),
                    );
                }
            }

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
        pub fn remove_did_from_storages(did: DID, who: T::AccountId) {
            // Delete from Lookup storage.
            DidLookup::<T>::remove(did);

            // Delete from DIDs storage.
            Dids::<T>::remove(did);

            // Delete from Reverse Lookup storage.
            DidReverseLookup::<T>::remove(&who);
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Checks if the given pallet and function names correspond to the `create_did` call.
    ///
    /// # Arguments
    ///
    /// * `pallet_name` - The name of the pallet.
    /// * `function_name` - The name of the function.
    ///
    /// # Returns
    ///
    /// Returns `true` if the pallet and function names match `Did` and `create_did`, respectively.
    fn is_create_did_call(pallet_name: &str, function_name: &str) -> bool {
        pallet_name == "Did" && function_name == "create_did"
    }
}

use frame_support::dispatch::DispatchInfo;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, SignedExtension},
    transaction_validity::{InvalidTransaction, TransactionValidityError, ValidTransaction},
};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[derive(Encode, Decode, Clone, Eq, PartialEq, Default, TypeInfo)]
/// A struct representing the `ValidAccess` signed extension.
///
/// This extension ensures that temporary DIDs that have expired and account IDs that are not mapped
/// to a valid DID cannot call any extrinsics.
pub struct ValidAccess<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> ValidAccess<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

/// Debug impl for the `ValidAccess` struct.
impl<T: Config + Send + Sync> Debug for ValidAccess<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "ValidAccess")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config + Send + Sync + scale_info::TypeInfo> SignedExtension for ValidAccess<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo> + GetCallMetadata,
{
    type AccountId = T::AccountId;
    type Call = T::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = ();
    const IDENTIFIER: &'static str = "ValidAccess";

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<ValidTransaction, TransactionValidityError> {
        let (pallet_name, function_name) = (
            call.get_call_metadata().pallet_name,
            call.get_call_metadata().function_name,
        );
        if Pallet::<T>::is_create_did_call(pallet_name, function_name) {
            return Ok(ValidTransaction::default());
        }
        let did = DidReverseLookup::<T>::get(who);
        if did.is_none() {
            if Delegations::<T>::iter_prefix(who).next().is_none() {
                return InvalidTransaction::Custom(0).into();
            }
        } else {
            let did = did.unwrap();
            if let Some((did_doc, block)) = Dids::<T>::get(did) {
                let current_block = frame_system::Pallet::<T>::block_number();
                if did_doc.did_type == DidType::Temp && current_block > block {
                    Pallet::<T>::remove_did_from_storages(did, who.clone());
                    return InvalidTransaction::Custom(1).into();
                }
            }
        }
        Ok(ValidTransaction::default())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Self::validate(&self, who, call, info, len)?;
        Ok(())
    }
}
