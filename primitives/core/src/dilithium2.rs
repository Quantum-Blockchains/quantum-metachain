// This file is part of Substrate.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// tag::description[]
//! Simple Dilithium2 API.
// end::description[]

#[cfg(feature = "full_crypto")]
use sp_std::vec::Vec;

use crate::{
	crypto::ByteArray,
	// hash::{H256, H512},
};
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use crate::crypto::Ss58Codec;
use crate::crypto::{
	CryptoType, CryptoTypeId, CryptoTypePublicPair, Derive, Public as TraitPublic, UncheckedFrom,
};
#[cfg(feature = "full_crypto")]
use crate::crypto::{DeriveJunction, Pair as TraitPair, SecretStringError};
#[cfg(feature = "std")]
use bip39::{Language, Mnemonic, MnemonicType};
#[cfg(feature = "full_crypto")]
use core::convert::TryFrom;
#[cfg(feature = "full_crypto")]
// use ed25519_zebra::{SigningKey, VerificationKey};
use pqcrypto_dilithium::dilithium2::{keypair_from_seed, PublicKey, SecretKey, verify_detached_signature, DetachedSignature, detached_sign};
use pqcrypto_traits::sign::{PublicKey as PK, DetachedSignature as DS, SecretKey as SK};
#[cfg(feature = "std")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
// use sp_runtime_interface::pass_by::PassByInner;
use sp_std::ops::Deref;
#[cfg(feature = "std")]
use substrate_bip39::seed_from_entropy;

/// An identifier used to match public keys against dilithium2 keys
pub const CRYPTO_ID: CryptoTypeId = CryptoTypeId(*b"dil2");

/// A secret seed. It's not called a "secret key" because ring doesn't expose the secret keys
/// of the key pair (yeah, dumb); as such we're forced to remember the seed manually if we
/// will need it later (such as for HDKD).
#[cfg(feature = "full_crypto")]
type Seed = [u8; 32];

/// A public key.
#[cfg_attr(feature = "full_crypto", derive(Hash))]
#[derive(
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Clone,
	Copy,
	Encode,
	Decode,
	// PassByInner,
	MaxEncodedLen,
	TypeInfo,
)]
pub struct Public(pub [u8; 1312]);

/// A key pair.
#[cfg(feature = "full_crypto")]
#[derive(Copy, Clone)]
pub struct Pair {
	public: PublicKey,
	secret: SecretKey,
	seed: Seed,
}

impl AsRef<[u8; 1312]> for Public {
	fn as_ref(&self) -> &[u8; 1312] {
		&self.0
	}
}

impl AsRef<[u8]> for Public {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

impl AsMut<[u8]> for Public {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0[..]
	}
}

impl Deref for Public {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl TryFrom<&[u8]> for Public {
	type Error = ();

	fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
		if data.len() != Self::LEN {
			return Err(())
		}
		let mut r = [0u8; Self::LEN];
		r.copy_from_slice(data);
		Ok(Self::unchecked_from(r))
	}
}

impl From<Public> for [u8; 1312] {
	fn from(x: Public) -> Self {
		x.0
	}
}

#[cfg(feature = "full_crypto")]
impl From<Pair> for Public {
	fn from(x: Pair) -> Self {
		x.public()
	}
}

//TODO Public key to hash
// impl From<Public> for H256 {
// 	fn from(x: Public) -> Self {
// 		x.0.into()
// 	}
// }

#[cfg(feature = "std")]
impl std::str::FromStr for Public {
	type Err = crate::crypto::PublicError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::from_ss58check(s)
	}
}

impl UncheckedFrom<[u8; 1312]> for Public {
	fn unchecked_from(x: [u8; 1312]) -> Self {
		Public::from_raw(x)
	}
}

//TODO Hash to public key
// impl UncheckedFrom<H256> for Public {
// 	fn unchecked_from(x: H256) -> Self {
// 		Public::from_h256(x)
// 	}
// }

#[cfg(feature = "std")]
impl std::fmt::Display for Public {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.to_ss58check())
	}
}

impl sp_std::fmt::Debug for Public {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		let s = self.to_ss58check();
		write!(f, "{} ({}...)", crate::hexdisplay::HexDisplay::from(&self.0), &s[0..8])
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

#[cfg(feature = "std")]
impl Serialize for Public {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_ss58check())
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Public {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Public::from_ss58check(&String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))
	}
}

/// A signature (a 512-bit value).
#[cfg_attr(feature = "full_crypto", derive(Hash))]
#[derive(
	Encode,
	Decode,
	MaxEncodedLen,
	// PassByInner,
	TypeInfo,
	PartialEq,
	Eq
)]
pub struct Signature(pub [u8; 2420]);

impl TryFrom<&[u8]> for Signature {
	type Error = ();

	fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
		if data.len() == 2420 {
			let mut inner = [0u8; 2420];
			inner.copy_from_slice(data);
			Ok(Signature(inner))
		} else {
			Err(())
		}
	}
}

#[cfg(feature = "std")]
impl Serialize for Signature {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&hex::encode(self))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Signature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let signature_hex = hex::decode(&String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))?;
		Signature::try_from(signature_hex.as_ref())
			.map_err(|e| de::Error::custom(format!("{:?}", e)))
	}
}

impl Clone for Signature {
	fn clone(&self) -> Self {
		let mut r = [0u8; 2420];
		r.copy_from_slice(&self.0[..]);
		Signature(r)
	}
}

//TODO Signature to hash
// impl From<Signature> for H512 {
// 	fn from(v: Signature) -> H512 {
// 		H512::from(v.0)
// 	}
// }

impl From<Signature> for [u8; 2420] {
	fn from(v: Signature) -> [u8; 2420] {
		v.0
	}
}

impl AsRef<[u8; 2420]> for Signature {
	fn as_ref(&self) -> &[u8; 2420] {
		&self.0
	}
}

impl AsRef<[u8]> for Signature {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

impl AsMut<[u8]> for Signature {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0[..]
	}
}

impl sp_std::fmt::Debug for Signature {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "{}", crate::hexdisplay::HexDisplay::from(&self.0))
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl UncheckedFrom<[u8; 2420]> for Signature {
	fn unchecked_from(data: [u8; 2420]) -> Signature {
		Signature(data)
	}
}

impl Signature {
	/// A new instance from the given 64-byte `data`.
	///
	/// NOTE: No checking goes on to ensure this is a real signature. Only use it if
	/// you are certain that the array actually is a signature. GIGO!
	pub fn from_raw(data: [u8; 2420]) -> Signature {
		Signature(data)
	}

	/// A new instance from the given slice that should be 64 bytes long.
	///
	/// NOTE: No checking goes on to ensure this is a real signature. Only use it if
	/// you are certain that the array actually is a signature. GIGO!
	pub fn from_slice(data: &[u8]) -> Option<Self> {
		if data.len() != 2420 {
			return None
		}
		let mut r = [0u8; 2420];
		r.copy_from_slice(data);
		Some(Signature(r))
	}

	///// A new instance from an H512.
	/////
	///// NOTE: No checking goes on to ensure this is a real signature. Only use it if
	///// you are certain that the array actually is a signature. GIGO!
	//TODO implement function from_h512
	// pub fn from_h512(v: H512) -> Signature {
	// 	Signature(v.into())
	// }
}

/// A localized signature also contains sender information.
#[cfg(feature = "std")]
#[derive(PartialEq, Eq, Clone, Debug, Encode, Decode)]
pub struct LocalizedSignature {
	/// The signer of the signature.
	pub signer: Public,
	/// The signature itself.
	pub signature: Signature,
}

impl Public {
	/// A new instance from the given 32-byte `data`.
	///
	/// NOTE: No checking goes on to ensure this is a real public key. Only use it if
	/// you are certain that the array actually is a pubkey. GIGO!
	pub fn from_raw(data: [u8; 1312]) -> Self {
		Public(data)
	}

	/// A new instance from an H256.
	///
	/// NOTE: No checking goes on to ensure this is a real public key. Only use it if
	/// you are certain that the array actually is a pubkey. GIGO!
	//TODO implement from_h256
	// pub fn from_h256(x: H256) -> Self {
	// 	Public(x.into())
	// }

	/// Return a slice filled with raw data.
	pub fn as_array_ref(&self) -> &[u8; 1312] {
		self.as_ref()
	}
}

impl ByteArray for Public {
	const LEN: usize = 1312;
}

impl TraitPublic for Public {
	fn to_public_crypto_pair(&self) -> CryptoTypePublicPair {
		CryptoTypePublicPair(CRYPTO_ID, self.to_raw_vec())
	}
}

impl Derive for Public {}

impl From<Public> for CryptoTypePublicPair {
	fn from(key: Public) -> Self {
		(&key).into()
	}
}

impl From<&Public> for CryptoTypePublicPair {
	fn from(key: &Public) -> Self {
		CryptoTypePublicPair(CRYPTO_ID, key.to_raw_vec())
	}
}

//TODO implement
/// Derive a single hard junction.
#[cfg(feature = "full_crypto")]
fn derive_hard_junction(secret_seed: &Seed, cc: &[u8; 32]) -> Seed {
	("Ed25519HDKD", secret_seed, cc).using_encoded(sp_core_hashing::blake2_256)
}

/// An error when deriving a key.
#[cfg(feature = "full_crypto")]
pub enum DeriveError {
	/// A soft key was found in the path (and is unsupported).
	SoftKeyInPath,
}

#[cfg(feature = "full_crypto")]
impl TraitPair for Pair {
	type Public = Public;
	type Seed = Seed;
	type Signature = Signature;
	type DeriveError = DeriveError;

	/// Generate new secure (random) key pair and provide the recovery phrase.
	///
	/// You can recover the same key later with `from_phrase`.
	#[cfg(feature = "std")]
	fn generate_with_phrase(password: Option<&str>) -> (Pair, String, Seed) {
		let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
		let phrase = mnemonic.phrase();
		let (pair, seed) = Self::from_phrase(phrase, password)
			.expect("All phrases generated by Mnemonic are valid; qed");
		(pair, phrase.to_owned(), seed)
	}

	/// Generate key pair from given recovery phrase and password.
	#[cfg(feature = "std")]
	fn from_phrase(
		phrase: &str,
		password: Option<&str>,
	) -> Result<(Pair, Seed), SecretStringError> {
		let big_seed = seed_from_entropy(
			Mnemonic::from_phrase(phrase, Language::English)
				.map_err(|_| SecretStringError::InvalidPhrase)?
				.entropy(),
			password.unwrap_or(""),
		)
		.map_err(|_| SecretStringError::InvalidSeed)?;
		let mut seed = Seed::default();
		seed.copy_from_slice(&big_seed[0..32]);
		Self::from_seed_slice(&big_seed[0..32]).map(|x| (x, seed))
	}

	/// Make a new key pair from secret seed material.
	///
	/// You should never need to use this; generate(), generate_with_phrase
	fn from_seed(seed: &Seed) -> Pair {
		Self::from_seed_slice(&seed[..]).expect("seed has valid length; qed")
	}

	/// Make a new key pair from secret seed material. The slice must be 32 bytes long or it
	/// will return `None`.
	///
	/// You should never need to use this; generate(), generate_with_phrase
	fn from_seed_slice(seed_slice: &[u8]) -> Result<Pair, SecretStringError> {
		let mut efef: [u8; 32] = [0;32];
		efef.copy_from_slice(seed_slice);
        let ( public, secret ) = keypair_from_seed(efef.as_mut_ptr());
		let mut seed = [0u8; 32];

		seed.copy_from_slice(seed_slice);
		Ok(Pair { secret, public, seed })
	}

	//TODO implement
	/// Derive a child key from a series of given junctions.
	fn derive<Iter: Iterator<Item = DeriveJunction>>(
		&self,
		path: Iter,
		_seed: Option<Seed>,
	) -> Result<(Pair, Option<Seed>), DeriveError> {

		let mut acc = Seed::default();
		
		// let mut qwerty = self.secret.as_bytes();
		// let seed = hex!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60");
		let qwe = self.secret.as_bytes();

		for i in 0..32 {
			acc[i] = qwe[i];
		}

		// acc.copy_from_slice(self.secret.as_bytes());


		// let mut acc = self.secret.into();
		for j in path {
			match j {
				DeriveJunction::Soft(_cc) => return Err(DeriveError::SoftKeyInPath),
				DeriveJunction::Hard(cc) => acc = derive_hard_junction(&acc, &cc),
			}
		}
		Ok((Self::from_seed(&acc), Some(acc)))
	}

	//TODO implement
	/// Get the public key.
	fn public(&self) -> Public {

		// if data.len() != 2420 {
		// 	return None
		// }
		let mut r = [0u8; 1312];
		r.copy_from_slice(self.public.as_bytes());
		Public(r)

		// let qwerty: [u8; 1312] = self.public.as_bytes();

		// Public(self.public.into())
	}

	//TODO add error handling
	/// Sign a message.
	fn sign(&self, message: &[u8]) -> Signature {
		// let mut signature1: [u8; 2420]  = [0; 2420];
		let signature = detached_sign(message, &self.secret);
		// let f: [u8; 2420] = signature.as_bytes();
		Signature::from_slice(signature.as_bytes()).unwrap()
		// Signature::from_slice(&signature1)
	}

	/// Verify a signature on a message. Returns true if the signature is good.
	fn verify<M: AsRef<[u8]>>(sig: &Self::Signature, message: M, pubkey: &Self::Public) -> bool {
		Self::verify_weak(&sig.0[..], message.as_ref(), pubkey)
	}

	/// Verify a signature on a message. Returns true if the signature is good.
	///
	/// This doesn't use the type system to ensure that `sig` and `pubkey` are the correct
	/// size. Use it only if you're coming from byte buffers and need the speed.
	fn verify_weak<P: AsRef<[u8]>, M: AsRef<[u8]>>(sig: &[u8], message: M, pubkey: P) -> bool {

		let detached_signature: DetachedSignature = match DS::from_bytes(sig) {
			Ok(ds) => ds,
			Err(_) => return false,
		};
		let public_key: PublicKey = match PK::from_bytes(pubkey.as_ref()) {
			Ok(pk) => pk,
			Err(_) => return false,
		};

		match verify_detached_signature(&detached_signature, message.as_ref(), &public_key) {
			Ok(_) => true,
            Err(_) => false,
		}
	}

	/// Return a vec filled with raw data.
	fn to_raw_vec(&self) -> Vec<u8> {
		self.seed().to_vec()
	}
}

#[cfg(feature = "full_crypto")]
impl Pair {
	//TODO implement
	/// Get the seed for this key.
	pub fn seed(&self) -> Seed {
		self.seed.into()
	}

	/// Exactly as `from_string` except that if no matches are found then, the the first 32
	/// characters are taken (padded with spaces as necessary) and used as the MiniSecretKey.
	#[cfg(feature = "std")]
	pub fn from_legacy_string(s: &str, password_override: Option<&str>) -> Pair {
		Self::from_string(s, password_override).unwrap_or_else(|_| {
			let mut padded_seed: Seed = [b' '; 32];
			let len = s.len().min(32);
			padded_seed[..len].copy_from_slice(&s.as_bytes()[..len]);
			Self::from_seed(&padded_seed)
		})
	}
}

impl CryptoType for Public {
	#[cfg(feature = "full_crypto")]
	type Pair = Pair;
}

impl CryptoType for Signature {
	#[cfg(feature = "full_crypto")]
	type Pair = Pair;
}

#[cfg(feature = "full_crypto")]
impl CryptoType for Pair {
	type Pair = Pair;
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::crypto::DEV_PHRASE;
	use hex_literal::hex;
	use serde_json;

	#[test]
	fn default_phrase_should_be_used() {
		assert_eq!(
			Pair::from_string("//Alice///password", None).unwrap().public(),
			Pair::from_string(&format!("{}//Alice", DEV_PHRASE), Some("password"))
				.unwrap()
				.public(),
		);
	}

	#[test]
	fn seed_and_derive_should_work() {
		let seed = hex!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60");
		let pair = Pair::from_seed(&seed);
		assert_eq!(pair.seed(), seed);
		let path = vec![DeriveJunction::Hard([0u8; 32])];
		let derived = pair.derive(path.into_iter(), None).ok().unwrap().0;
		assert_eq!(
			derived.seed(),
			hex!("7fa53fe0aa2d4b4bb163790f6cf56fceaf6c6208242aeb8670f71c98611ab637")
		);
	}

	#[test]
	fn test_vector_should_work() {
		let pair = Pair::from_seed(&hex!(
			"9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
		));
		let public = pair.public();
		assert_eq!(
			public,
			Public::from_raw(hex!(
				"b2667be25db6ce9acc69cc6b4b72e67b61c5a1dd280a1af849682bfac56aa622885c94477972721683981cc47ade25588dba5069877a113de7e6820210c099015380ee3ba1cdd7e3984777e36acf06782dfee6f3d26d6eff96f8a60c51cb05c67ca45b671d29387d35e313bfd24ab3a8b9f29ccfda18bf55c11738cf69b87f64f66c8d3117c397156ae1e1ccf0b43b0160bfba691faeab59ed483f7586849fb2122f60ed6e9a48255522f2de543cdeaca1689ae91be8beb9a41f7caf31973cedac19425fe570332bf883bf8ce0ce1c395bcd22d56df3c838f6b4b0d1736b13070118422447ecc77ff2ba0749d512342cd228c4a2845a3d62f37b31e16bc081d1a42595c31ac0650b974c78e2440abd249477a9d1d1f3d10f497a1ad827b7e9a4a149cbe560bf195a731a53457fc4b7915706696dc46c59b8f815053c612fe0f3464689d9490a33cd5d9860c8061ea1d580f7582c783b6fde0af715a9686ad27139b8979db56d0b3f66efd96b14b96081098782f18cd925144362ef8847cb0621553f0fd22b8f8015b5791768133d37aaca84e6214ebefe7a050833075f82ed1f05e099228de7761bf47e6ef21688243ad841231f197ede6fcfdef1e24b6a2a420d246014006d831925c24b9fa7b1f74582da8a1fb438623b112dc45bc1759f03dfe2e2f51c18a8e0d666ecbcc7b0354070a2a03547ad74d2f5a54239c548bb9b3732437abae94998997cf3d2f0882d66e0b981d353ec3dcca7584e8cb6f4031103c40c709341e86a7c3a70eeb641f0a3f147da1ef44ae9b9c41ad2e1d0370e746ae42ca49d934060ce251e4782596f215c473b755822c3d45d44ca1745c39db172f3436570ab29fdc4957546015a656cc20d66137200e05a57000a08709a6ef10e7938ca72239e82e37365370bbbff4b98d02b891c4ce0ca337a3875308fc8db119807e08e82e0e4ddfb350aceb2ee729373a4cb68535b64098553700d80c4185a5c2e3444f70eb168d3180c4c1869d5139f1f9d82227463ea7cd854a8c708954220a6209d481cc166d5d0a96e07831e247b3272a6b2310853a0b14fe3958ddcd83b05ae6663c2f3b5537f910e54e35c96d4896e758cdd9ab0eda48cc72c9b6417853b1a3e6dba180314d4454c2b0532489bac25712e77e3f0ca385d4d99d4cd974ab40e2fd0a3011373e5edb6e20a5f198fa23e740100eccbdd7318b1daec16a6d37b246e4e51ef10e14ae4ddd13c1a8882657b4c720dfdf609a0ad094e47dc6fb87dc4f75feffec48ec2853498deadc1e298d26b9953a6f6e48d00b22dc1aadd661a59f198e1c82e0b8a9facd10a4f8cb2115715ce0835104db86ca8112afb2aaeec67c06bcd0bc8871babd085418410c519df9e3a8e599a72657e804ce6c3d35039ab665b10482a1e1f0f3ba0662ee6ae3f0ce86dd702473181407385969f24c201f125daabc2104212feef57aa445c3c77f7afb7ddd1d33f6fce633f7c8b7a5db2ac35fb5c584de9b3aed47e3a3e9dcafd3e2cb2ecd026c7570fd2ecaa185e97fce6de8cf5dcc50de3cf6010a49853f26639b2380545de200f4a9f2f178c26d0c367a8afccdca334667367c1e8030194bf5198d746fac7b7d885f0bff406700db83e7f2d8cff65b5336be078194a963776cd171a34b32518dbc8bb91216b086421272b400d29f2ecab5bbeea4366a4fbb25d2830a5933507cc73f289a09fac92f6754a5b7aa417b3d63d11093872cb48b3b8f0dfcb919a5b1e513a5c838be3a8a7fba0ea8fa44f1e52d08d1e349de9c98ff3e1935674b253042c9b5dda7cfcdd889d6f15fab93a2d24437e6d856b3149e05ddcdd62e37f98d79f6ce88ff1"
			))
		);
		let message = b"";
		let signature = hex!("803182ae2b5ffc1307a935dfcc314ad0a5028c262068d0b13c1eea8423218e2cce331b774521c3230ad50aa93d81d4478cfbe0c15dfb4c20ea600c0558c32801c2a0e1133a274b63b7ab50e183539d22a957cdfd1490695eeea2a1eabca4342dded16eecc5bcaee0be77d85253f3b75b3e019016505cf9e084b714f4e9da82d3e9501feb90da0bad1dcaad0445717004ab26cb4c0d357f36d31d5d258db5958ab388e8cfcfeaf0fb7495460d48b811782620ddb42a33228c5368120d322c259001eea26b68ad4334c9f82902160df63957ebd650bceaa4d67ff433cd6594a1e5cad59ea7289db8d38c1522111a0b2db373ebb08b611f8183f5222a326599b34dde95d28848b38ec866be3371c4095bf5418c8294fc656c66c1a1c78a1c597f5be3d3cddd41b74e2cf4e7b12b9c8c712ac4eae38aeca754cc1f74aa1b3080e8765bc0ba060b8e7a8863b58d56fcbd5e26f86c1745aa17de06ce0bbdc45848a7519fd6d4004983fb4e0330f38f4d683606042d78c07939c1a4fe1d8fc6b1421bcc4b711f929450d0e4386ad253b22d4d23506d2e62bbb8589737438ed1df2423d63cda7fe663666cd7dc2167e338c6ad0c92d0a267df05eb435a95f50ac6ec150ea7e299c04ae7feabbd8370fd4ad1c485543900c8205d3656e8e794965d143773599b24b2fa8b8f74d13d31373ccf18b037a0e7aeaa1932086143e3675b5a59c997f46a6e3cf2ba7f47f819d89da8f5bbcee81ff04f1a2b3ce823e4fbcfcb29ac1a79115f9c07fba77cb615692eab9de8b1c42ee11802bf737ba26e258ecdaaf4dc7e453ae42de51ef083e70f51a39983842f4d6ec4944d299d81364c240b531b87d4e5916949fc4a736825865004a9a26a4239f76b8f98757c085efe232912e47670414e625f685e94d5b1d589b1261da05b958e4b750f50bc81084c3352b3a74262f65b70d2986ae34fa8cefefdb3a79c13b12b8cfd8748c5c80717ffffa38f40fe6259b6e829990868820102bb3997250531aaf9ec80af9542099644bcc8e3bf53b891b835948615e9a0de30c37f16a798d8ed1688a39ac386a529a4c8f1c694e77b7e549dbbceb894fac36ed149a48e3c6d7b235bf01081ff0e7460f2906c118ac204337f0ed1746b05b9dc1f9278662373339b55d60ac674d19d6ec5bcc8943a6d38349eb4b1f7274f0f781158cc5781f5470b03db9983f78760158cae0512ca7e956bad73402040cc9abfff424f328ea883eccafe1167095dbb6f070db3a092bcbc45787bc8110869a288771561900a4952bb149339071b6d84cf01e4f8ae0d5f79ef4eaf8f84385af7307510f0f220f75f4193ffb4bc74de5f795a9a03a9498a47c0b367327b2e0308c783238c767f447ee92da4b51a1b2c49643deaad82bfa7051e92aa4084b8a69d5733bc799c2757c4f64a673796a1769046bb3493cdb4eb7d7bc58e7310d7c7b59d070b2de9ab50bcec9e2d2201f7868801011a57b7f941ef8e03fe6f5b7cdcc3411ed7ecc9e81ba715c04d370837c0227d89a47a4fad251dee7d10811a1fe477f5da15700a7140b3ca4f2bf4a8ca8f50fa8c01fce6c2b25bbd99ae541cf4a2a7c322641923003ca68a478a04015cd2cb80f9b90f5845777f7b16dc9ea082f33061c6d62a48a71039a55084652b04d95c7a25f232177384268f89b1e61ae7ed0ae389029f059c2acab62eb9d788fc06e6520046a4175a4fb03e54dabc43ce87d18ff09cdf024a9f4aa8e7815375033d5ae7f0a1eceb9a2c9750071476592504e85f5723455a9caebdd5c889715cd6e7053a7ee4086050cae2b02799779af1565152e286c5fabbbadcb43267ba75dc3e992f5b4d773472400a73eaee67abc4098024d3c065a50243c5f45e83cc8750bdd04df26bac31df72b8c06912e684edf8c8073e6e91f0b5fd8305072e0ad0997c3c55d68d558a9b0317cfe865fa3053d420839eef70d256049ae7146ae60f9cb9a5a31d11fba5e35198dfa6617d88f8cb8c759ee48981f726df3e5696e2d8925f1d2abc39b3695ba74e4aa2de0d706d668220c1441da5cafe1b889f54e2d6d8d9c16ef356147981c760b3437543ca881cda359557b3652d8b2526381280fd8a0b641878b7425275accf77515cfaa13efb327d225157e02adb682038dc522c644673eeb81c9ca5a7c58f562eea177c99d650843920c74adc09c86d278dd36a532cb4a016ba6b769817f04610159706ffe1febd940cca04123be3ec04bf63a9db07bc19ea71de178792a221cdd70677cda70ea4f9d737861fe3c74a4ea9b77dfeb0a37c8bfb7632a5b2ed90c52c7bf4d393bd84e728bb18d0b878b33093c6ea8a2055c487cb54913cbdc5b2aecc02c571c27f271e4adfa735ed3b45d3182279c7794e76f82b1b61d61e992138c6b8bfd7f5df9d5f920105ad8ff533f304b41aa4e16d47d00db8f5307ada7d510373af6f77774c00567c459d64abf7c7f27f9d8fa8b3774571580746776f3230732502802e56836eae86386b7769da236863105879acfceb78ec706bbdecda6730fee35b397df8dbc3334c29a9d38b5380874e05759ec4d52a6242e5226662f827d9f69f3980649076f83e179bfe861771e2055a6057c32c808b57544ecbeb74034fc64ddb78411e6f005fb8c0b7b119402f7d455b0ad372478ca06b179f6b1bf50f3e3b072c26067b7f551226157358345b0db896c292ddf9c881121b6c8a283e116ae4dc66d4ac33e64639feed93dc45a7f5a386870c209610942dfa506163c6ddb3b65249692c153ac319884dfd063e73341aea5cfa7cd47b2ed155a66926f2e8dd8067c80d538cd9f615cbd997c1cd05c18dbb85b4ecbf10bc3b23e2a887bbc763cf467dd319c619c432ee469d8764138ad569e02c3e3595e5f1752580bef491a74f25cbc9be590f55af6ad10e8071e69412a6a6c608080b7f9a73b3ab07bb3a3882c374ec19464635a79f8aec31d9143b165e5c7e9c6a7ce9ab0300dd88cd6458163552debed1e01f3ad17815ead7087897311efe3b1d7adb637a1d6b76ee99a3efae25d91f2e4fc96618d4099ee9310946b6f8e8b4f9739804e9028961fc2907a753c3303469682c19a215f5eac3288f6cd23cb0bd87bb1d53a3789da3858c09163c691a680929d683a8dafd17ffc996a1dc577391fa4e90df833be9410d5187aceed68feed6367f437a2b8e06f2c7f27fcf12869d2b1752f5a71adf9595116a1784cafcfa1a79d8f4668870e2b4bdb1ef3e3d5ad5dd91befdc1b20e2637b53a7a153b9fd6edaf2c154f1eb41dde8f3348145c0d2b747604192c3239464b636f93b6bdc2c6c9e31b1c265a60646b8693969facb5c3d6e1fc060713273e5c639398b0bf020d2b395e7f878b92a4bdbec9d6f300000000000000000000000000000000000000000010212c3b");
    	let signature = Signature::from_raw(signature);
		assert!(pair.sign(&message[..]) == signature);
		assert!(Pair::verify(&signature, &message[..], &public));
	}

	#[test]
	fn test_vector_by_string_should_work() {
		let pair = Pair::from_string(
			"0x9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
			None,
		)
		.unwrap();

		let public = pair.public();
		
		assert_eq!(
			public,
			Public::from_raw(hex!(
				"1a11b9c2bdffe28e50c19257de1eea4acb4ec87c053ea01eb90428ec231ea78baeedca123c9f624ed3a6f166bea6a9aa36f709e7487de33eb6d816172fa068fdc03000928cfe14d651cb9b4e48f57153698a84190e876cd5af5eed55efda65e103aa412b18ea27df6a169e0ec9fc13b90e1b148e5e125787a9a868b8eab6bc69db2a0a426386507790f91630e73cc687d68ebbb5754895b3342f13b8b0704aa66df138b30493bf7c0b089cedc0a1f2e4243f5e7f7e699f2c74a1f85167681ae72c8afa8489cd3f126f992ce02e7936b24c28b348c2fcec6f9f19f56dfb8f0e33200b2ac2f5ec1ac3dc301d5ac4c71dbb120a1c0593b41733062a2f9876139a7fcf8cdc22fbcfab1db3dbb62c01ede753faf1e68d3059ffad758b70baa654b5a82b5b9bb63ac70ead7eff5e77a303e9aee23ec18b3542b4fb3cd43af386c4dba2b0fb2e5484ceb7bddd03fa28a02965b14a43dc2d2c990d198b3d92392fb7ff42a54fd9fb3d2f2a2cd420b070d4f66bfe636256c188e84cd818f9668139d3bbede8fbf883f572f944f8a1810da1ba9681e488750054a042a06dae46e62c308087817d50e62ba7592bd94daa59f27b90b7d89f1085cbf325bdd745a1643d5071ce356815d228f70d680f0811b9c6cdde32849c4ef35ac907d908f1efa66830e9a958e2ee47d58c77fca9098f6253e7ac459ffe81c2709ad25f75a1dfde398588ba5b69f5d75c69096d6ad86534bf398b3391e3327c127b8da047e9aa51f80efd0a832815fa6a87ccc573bcaa9f57cab17e68310f3a0cd875d3a5a4c5a0e39a5d2eb4e8035c25348e4f26e41b5ba9b3ec59467660b7daa03069ea2a190b488a42565f712ad82b67876dedda46e5ab2c45e8c0139be024b42d3f908549cfa1c406270553a7bdd39a71bcf2e50636615414f7165bdfe7701329f1164d806d93d2f8c347894cf49408f18d12d35625eba33d86fa5a76949e9226df94801dfb378de848540918c8b825483abe4ead876d9663a9d49de5e061f0ee92596de32a6c8b26f3569de138bad4a7e9504c96898a9634617eedfc417b3b7ce15d4e90fb2b228c8c3e584215c5797aae0b708a374af8a759bf410af60ab942d335c296e518fd59ce75b9e46136998629555294f723c69b6ce773ce5e1384e81191f4ab5bd1dd19ff757e0a966394a1378abc7234e1e40f68219576467f9cd358e4ea378a22cf869074beb9b6d2e8b7f9bb58b1887a9841bb25d575e162cdd37fb9dc987ebac88766a3f92bbb90fa8e46cbf8a2508830a3458c3bde3ead8eae9b24b04ef19962031848abc96cc62085903f3bc5624f5c7a9fab863d16f1c1cf4cb9173b738a6b6681e23b9122b597486143f7f9c18d4ce3d6dfd958970cef02315c71beb6241c60dc25fae4c1a717cc7d5852ce53c44f641c6bc13203f30db412c7f2657489654a8754bf42c5394bc13d2da096c1337d08d709119ed1f3e420ea9d3fe8449620237e714329478eec414da6dd4dcd972d3eed69ed549084215aea955ee78e9a502b18c01df4b1e62f25f25dd6375d3ef4bd3baba22abda2db7772bd85ca18aa25f2904d6a6842c9fdb44a4e06524faa3d0e24125d6cc8d7fae62bc681357de91104650612e6ddf4ade0c8a6e734eca574cca0ae2d9dc01277451ec93ba88071b12ccfc98afaf54cd0016f517917b3259345cb750b2b44a106f665c3503c75961d6d5e31d7e7f20f7e5d02f15e98293a1d5fb15d61b834592b2079c718b2721cba3b7168395aaff0b926664309e3060030cc0a9c59bfaee9c89db0eb8e349e71e59f03b823d58bd28a6c783f829083ddb5bcc628b6c5edbb012e86d8aa58e416608361"
			))
		);
		let message = b"";
		let signature = hex!("6e5ac9c6c708bc4186e8142e628c6a7a1553d00e61665367a226eef5c15f95f5c842a2fce38f11138ec30104c91ba3899aceb265958789e687e219b4169701253c80349491a77fa8d510489f8a5a775998c74b3a6a45d4aaf9159ee9a757734598630f64980fb9b3e6bfb391b4c0d28227f42417737bb39c31fbf5e3d1a02937b6c6e05992fd55b50ab98ef1fd35123717b5e8346a61c3c1e2eb3f3022c5545e3a1c85b11f2d9814a5ca5fe8c33686e3a203713fb4bbf8f37ab94cdadb51965c1a027cab7ce094393a2e7703585af6508ef4e66e1608d371096e4eead42245ed1bb6156cabd6b93235010b01232639575541d8f6a1d7c64ca9dadf91fadb9e510e2b979794a41f2b6949a0a9f43ac6a07b8a2f6c486ee7888664232b1eb3f78e87d9565ae78e9771742e42adc45bd2814b6e138f527617935b6fbeb92969b1d1e6ac226b9f74e63c2c42263aa35629681a97a8201f4af1e586facf7b593e3f46e4c355fb0311a140712878d90b8cc1fe294bb05b6ecd83c37fc05527105084eeb0e00fc9a74e7a9ad1f0deabfd25cfc4767ab45bbc7a3d70ef82f137ddd794d44b180972add73b42442ba26762639e5a7b142d9e2d83124bc763288ea9f5919c4e44f256acf8e4ca997cd48d1656f40b7c335b58e1b83c2f33c45ebbf655fefed95f2501c5a72b8466e446b6f527e24b599ca116fe214a82d5e1fac9ba99d5f4870493b52154f0afaeb9e84fd10bdbd25d6d3fd8ecd1cfa5dc7e56925b2555215d8304ccf542001e3198740c881f4dba23803103e4b968fa98f6076de098d19e88e86347b0ea772cecbd2724a8645a959d107ddaf7ecc18e87a80709c9faaf29840911e185c906911a4b852b7e481ee6f6aab7d12cae717a1d8e7cefc685a158e4c99d58847c20ffdaaef59b31aac4f8571e30bca5b485aa692fd08dd5c6363fde9924b177fb8c8591d2371dd02f5fdc9ec35704f2e3b7e6ea850f521a0c652d2082ef696c7cb9c8fce6cff135b4e9580b4555a77b9702567c5a8d60ff47e2b61f5912a0f7b588becefd127f20826c1e2d16f59ee3fc7090b99cbee576d02b7fd195aa4b1cb11be8aec979448b3beb233fcb48d7797b1a3ac7493bd6ce96d85f64a45f7e26264f1139fb3a947f3913a45b1092520d7554bc1dff25123e8ee238c05995a7469e853557c06e9d89241d5c953a8df9a7b30431e26a26ffaab314a53110fa928dcd45f5aec74f97659a0ab7a035eb88cb7b5a60b4da1fde6fb096f680816f435edc7a07181e40b9c6c53468a0a65e0c352e4aba3297e85ec06f8bea4bed1235aaa242f81d6f08573e2c7aeea0910029a744c0296a37909ac889af22baf9807cc671a0e77acd6a0691848fe1f06e561dfe7b33ea5650518aa70c419214401957752ff9ee70ef43dcc073baddccd44cba7300ab98d3cfb82acdc4303fb7a39ff088a1d62e62cc6d4b634eedc10835980b9fe1fa3d39bdafff57563173ada46ff29fa0bd3065b2f7077a69e3c2b11c090a872a7bb2e628f13d4d0468da9dd106398916127bdffca11900a4b1911c7406d24538dd966d9633a08dfe8bbd0832caf7c50c1df38e2be6334c94469a6bc261bd1aec2658ae4f027341f0d0038a51dc8f5fe9652ee0e34c273d1a5df5c6daf1746b5f1f00ffb7fba610a6efc180b3e9974147265a241bffa2ed72f3b425cf179e389c59a48d3d8550bfaa449cadaeadbd84092c7f7be802c261417963bfc1305b26bb59ef5b091bc4598b35202a48b814dbaf9c651866aab59876df32ae79da1ad2c6fb8c462af3b1aec80e6e48bd44c095621da7b1b893887bb47bad35785b02e20e7952b23c77db938702040228f2f295dc0167170599335a888871b8c6c63f12589fd733317b3aecb151e8e38b9b3623bead056563ecffd24d325fa06623e1719b1c5ab100f789fb4f5ee17a70712dd473a1d375f6f8c1a0f5cddaa328919c278480b864454884087cfb08d926cd2e3cd940a7838f65ac6cbc9af19df6d3800465b997582f872a14c545e2d5e03705c50fdf10d7a85aad0c24f8be9956b5170303674561f6b9cc8e454c57665e1fae9478f492e79b9da031ddbb3f6cb4225449ed5cad82606d99f23cbacbad3dca7f1decfb037db05929697f68ccbf58559b2b5126dcbbe610ef8612512b6f08e7772eb6fe64c0a7d408ede32d2127f62689d16f2d779baa3dd24a0c5198c0b9c196604cf1903f5144b73d8d2c8a77b71fbefd8928c32ec7c3d41c6f02e752de26b2da6d113b0ceb654f6bef0d36400db969bba24ed24cc6fafa32c7fd756663bed8aa953e2bdcb3ae9e79cc753dcdd60c64cf667aa9c7f662d7dba60849fa4bb7e46771c13c94a77739755f43de27f41ca5cd685d7ff53b5cd5e520aed6d1fc763517fc9dabd632028a33aad740532f6b735fd3b4bd1a4d8c14af14690cc6120af316b0eee49a655623fecd52171fbf01f8cc4277f9a4289e3bca3fed73b8035efbbdc510efebc8aabed3f4569f7ba24546ab19150a287255ab8f512fa5743f4f348362ef2bfa887358d4fe9bbd5cda8a359f50478160fe318bbf0a4294ada7996efc9863f6fe6ebc906870feee785338c23f1645582f0ed9ed859504adf12411ddb8d3124e8602e44f2bfc438a753ae02516cac2ca9837c132a9599600921b01899fe285d09d13dcea16e918f8f798b9d742b42f8d24bf2dfd9c8672f2ed2cf7fc676b3f8d4ab7fcf3b7abdd2e51734537dff50df7428e9b5f918248e4add1ae690519cacd910398c836fc414b18694ecca89c8e12dc4f8aefc182f6845fa79660b56ec9e5884ba005c603e359f3230cdf070e33b4c948e17e8cbd673939c45a790ce11a6b007ec5471a0dbb15d099bc5deeb1004f16423393f22f5663a413a3ac32031b7922128494dc361a75b0c08728f1cdcc45d20595932bccb071eaf5f0fe2141e7334a2d615a8d28cbc811ee401077879685b6d1df3113404959fafd75c05da63cf7ad2447b73468b749705ef61c083af02cdce7363821a6bb2c5297aa152cdf9554d438b0faca127d8423b812727e603704225a30542974adbae630e7e98783a31ff0e01727eda1490f88ec0cfd417c8d434046b0b93011ecda5fc3bc4471d8e8a4d9539cdfc68e48ad437176d9f196eff9ceae2a01c0fb5bc39bb8dbde7e726472281bea41771499f88059a5c1741d2fb50eb48efea10d4ab6f4450bc7091e442d39b789e34ab24e0c19aec5d380c09f677014de35af614032187be36cd8d2145488dbc008a051c86ab3cd05c50002b256e8ccb214451267a50e181d2027282a4a7e88a7c1d3e0edfcfe040b55676f74969ea2a9cadbe8eef5122022252a3c434b4f63666b7483b4bfc8de02151a2327314a5a6d7077a6b7c3c6cfdce700000000000000000000000011203244");
		let signature = Signature::from_raw(signature);
		assert!(pair.sign(&message[..]) == signature);
		assert!(Pair::verify(&signature, &message[..], &public));
	}

	#[test]
	fn generated_pair_should_work() {
		let (pair, _) = Pair::generate();
		let public = pair.public();
		let message = b"Something important";
		let signature = pair.sign(&message[..]);
		assert!(Pair::verify(&signature, &message[..], &public));
		assert!(!Pair::verify(&signature, b"Something else", &public));
	}

	#[test]
	fn seeded_pair_should_work() {
		let pair = Pair::from_seed(b"12345678901234567890123456789012");
		let public = pair.public();

		assert_eq!(
			public,
			Public::from_raw(hex!(
				"794e44972440671aedbbec787b9f73e7c0f51d8bd0f7de106ee907e6caf986b12f2094222fafc7d9d301db73d235e4bf221f3b6d7d0657940b5e9706c4b5b8849b94777240c23bd5a8106b778e9621b79f0cad772e04e1e722d0cef8484c41ca0692bffe92303160e5b83236287259fb38ecf595002fdaeb08c62c0679a8dc21e788accf0cdced10a8a4f53b9ff4130e4f1bde74d2bbb33600cefb62fdc7748f172524bc3be6fea74028785f64d7bfeba22d01c2dc10a32d618afb2077dbff4635093d9380e18bfa42172d91a37f3e972fe161aed0edbb7be63929456cfebc37686107e6f75d717d77d14f29cca08e989bfe86f595c6f8b188c46c30e0f998f1a469a0268a0511ee56353e8205d8f28c0cee6cbad31a8fd73dbf08e27129fd47bf7943475d7b247b2cad6414366ca26c0ccd7ef6aed941028b766b1e2863274936fac9cd3eee2734c6e932208ed3f1d389dd727439c4ac33338597c43538b611d0db490e4141f48f18c56458c22bd3724b0d2a7c5d8e43b16ce2d7caaa2d0914f0fc04fc060ec23f43d6ddf4e10b1e6a354798af13e657bf2c91f23ea1f38006fdae081fbe83396109bd700fb9111246332857a3f4290d82c0e3b6a93cea175a4af45acc7c98c1b6e7787f5ab7fb89082769b5906f96b4f8a38122ef92dde39efa155d73a5dcf5ade4191dc62be27203999bd2b3457e8796deccfdde4274afb31d7c3211c75c2254ec810812fe0a118ecfed5adbbadf99c069e705173d751f4b6b2893facdde44efb16bd984cbb4d138858a59bcf49bd8c1bdb793f846f2cce41701afafb2a85314ecfce792daf187409ab877e6fe44689c874e306bb1370f6708bf74e23717a4231109dff9cac9842d0096d92fed6ba58babeebcc1d18d59d35ffeab8d74b0ccfd3af5cebb1b276e72c26a298c6dcb8f1139548b6e8699b1b9df5d5e847e345eebb6cd9821463cd2c75bc4bd9a5fe658cb2cb63b9fa55843dbc9c33ce172b9fa48c686e8e52bb3f7bb5d49ef58df2d7e9a9cf973158ff15662587d496d5b714e5376ee7f1d34eb4f1ac525279ea6090b88ea83878c3dd7c5f26dac6fdcb179352751abec1aeba9212fa912e912c726c9f70d73cb588a5494feca17228dfad02de4530b65dcad1bca1dc74a21cf01728c6545a1bdefdb540c394f6bfb8066cd55acf4c9a4aa6fd3995a9880729010f848829fd2cf526e075880aa5b20657c3fa3b12cfe6e2ec562adbcdcfa86f8778dd8d157eb9000615e43eb8a42e198d4591a9d52aba66a5b02ca73d022682ea307e5d95aaef5efabe21aefa01beb8644d71f9c3964c503bd98cf6d2bea6b2c8b147464a6f905d8069563f14ad9b7f0d6c55e77662c709f40985093d25e538e0d77befbe7de026536179f7b4ec89b8dfc0541b7a7ab541181c6be30d01d5acead27d5f1c275fb60ed2b6dfa77635f2cf3366c380fe003024d036e20aab6d280f8f6a1349778f8aece12eb39e60b82163ff67c76a062568f24eaeaaa0626eb6c2141e45b62a1e45dd8cd20f89dad93d4c54142b60eaa425dbb80a3e98bad24240f7d6e5a3dfab01060cc113e86c830a956554f03c8b51d67635de94479eb8e58601945a7f57e3234d6290524683d05bfb9b17e6dfa47e9c70499a4a119ccff62efeb69666c85cd7c5ba1ff5ca5fdbd21da018608bd66e15568d194e09d8e40e92b11382d48c58211a7063c9fd6346453fddbf6ed0b8dd06d099110c8e5f8e2659588d1c63588f1df66599b9d33c8aee2eda5c5c1d2486429ab2a308d66b9bac27b84598cd6649a54a20df6c6a7a7b80830094ef2485a83151e3b4fa419361afc6046012973972aa41b0e2134"
			))
		);
		let message = hex!("2f8c6129d816cf51c374bc7f08c3e63ed156cf78aefb4a6550d97b87997977ee00000000000000000200d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a4500000000000000");
		let signature = pair.sign(&message[..]);
		println!("Correct signature: {:?}", signature);
		assert!(Pair::verify(&signature, &message[..], &public));
		assert!(!Pair::verify(&signature, "Other message", &public));
	}

	#[test]
	fn generate_with_phrase_recovery_possible() {
		let (pair1, phrase, _) = Pair::generate_with_phrase(None);
		let (pair2, _) = Pair::from_phrase(&phrase, None).unwrap();

		assert_eq!(pair1.public(), pair2.public());
	}

	#[test]
	fn generate_with_password_phrase_recovery_possible() {
		let (pair1, phrase, _) = Pair::generate_with_phrase(Some("password"));
		let (pair2, _) = Pair::from_phrase(&phrase, Some("password")).unwrap();

		assert_eq!(pair1.public(), pair2.public());
	}

	#[test]
	fn password_does_something() {
		let (pair1, phrase, _) = Pair::generate_with_phrase(Some("password"));
		let (pair2, _) = Pair::from_phrase(&phrase, None).unwrap();

		assert_ne!(pair1.public(), pair2.public());
	}

	#[test]
	fn ss58check_roundtrip_works() {
		let pair = Pair::from_seed(b"12345678901234567890123456789012");
		let public = pair.public();
		let s = public.to_ss58check();
		println!("Correct: {}", s);
		let cmp = Public::from_ss58check(&s).unwrap();
		assert_eq!(cmp, public);
	}

	#[test]
	fn signature_serialization_works() {
		let pair = Pair::from_seed(b"12345678901234567890123456789012");
		let message = b"Something important";
		let signature = pair.sign(&message[..]);
		let serialized_signature = serde_json::to_string(&signature).unwrap();
		// Signature is 2420 bytes, so 4840 chars + 2 quote chars
		assert_eq!(serialized_signature.len(), 4842);
		let signature = serde_json::from_str(&serialized_signature).unwrap();
		assert!(Pair::verify(&signature, &message[..], &pair.public()));
	}

	#[test]
	fn signature_serialization_doesnt_panic() {
		fn deserialize_signature(text: &str) -> Result<Signature, serde_json::error::Error> {
			serde_json::from_str(text)
		}
		assert!(deserialize_signature("Not valid json.").is_err());
		assert!(deserialize_signature("\"Not an actual signature.\"").is_err());
		// Poorly-sized
		assert!(deserialize_signature("\"abc123\"").is_err());
	}
}
