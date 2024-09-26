use sp_std::vec::Vec;
use frame_support::pallet_prelude::{RuntimeDebug, TypeInfo};
use sp_core::{Decode, Encode};

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Request {
    pub identifier: Vec<u8>,
    pub operation: Vec<u8>,
    pub signature: Vec<u8>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum NymRoles {
    TrusteeRole,
    EndorserRole
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Nym {
    pub alias: Vec<u8>,
    pub did: Vec<u8>,
    pub role: NymRoles,
    pub ver_key: Vec<u8>,
    pub ver: Vec<u8>
}

pub type Dest = Vec<u8>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Schema {
    pub schema_id: Dest,
    pub issuer_id: Dest,
    pub attr_names: Vec<Vec<u8>>,
    pub name: Vec<u8>,
    pub version: Vec<u8>,
    pub ver: Vec<u8>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Element {
    pub name: Vec<u8>,
    pub value: Vec<u8>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Primary {
    pub n: Vec<u8>,
	pub r: Vec<Element>,
	pub rctxt: Vec<u8>,
	pub s: Vec<u8>,
	pub z: Vec<u8>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Revocation {
    pub g: Vec<u8>,
    pub g_dash: Vec<u8>,
    pub h: Vec<u8>,
    pub h0: Vec<u8>,
    pub h1: Vec<u8>,
    pub h2: Vec<u8>,
    pub h_cap: Vec<u8>,
    pub htilde: Vec<u8>,
    pub pk: Vec<u8>,
    pub u: Vec<u8>,
    pub y: Vec<u8>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct CredentialDefinitionData {
    pub primary: Primary,
    pub revocation: Option<Revocation>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct CredentialDefinition {
    pub cred_def_id: Dest,
    pub schema_id: Dest,
    pub ttype: Vec<u8>,
    pub tag: Vec<u8>,
    pub value: CredentialDefinitionData,
    pub ver: Vec<u8>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RevocationRegistryDefinitionValue {
    pub issuance_type: Vec<u8>,
    pub public_keys: Vec<u8>,
    pub max_cred_num: u32,
    pub tails_location: Vec<u8>,
    pub tails_hash: Vec<u8>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RevocationRegistryDefinition {
    pub rev_reg_def_id: Dest,
    pub cred_def_id: Dest,
    pub rev_reg_def_type: Vec<u8>,
    pub tag: Vec<u8>,
    pub value: RevocationRegistryDefinitionValue,
    pub ver: Vec<u8>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RevocationListValue {
    pub prev_accumulator: Option<Vec<u8>>,
    pub current_accumulator: Vec<u8>,
    pub revoked: Vec<u32>,
    pub issued: Option<Vec<u32>>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RevocationList {
    pub issuer_id: Dest,
    pub rev_reg_def_id: Dest,
    pub value: RevocationListValue,
    pub timestamp: Option<u32>,
    pub ver: Vec<u8>
}