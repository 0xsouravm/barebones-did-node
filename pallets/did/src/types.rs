use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::sr25519;

pub type DID = [u8; 5];
pub type DidMetadata = [u8; 50];
pub type PublicKey = sr25519::Public;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DidDocument {
    pub id: DID,
    pub public_key: PublicKey,
    pub metadata: DidMetadata,
}
