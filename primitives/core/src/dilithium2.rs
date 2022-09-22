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
	hashing::keccak_256,
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

//TODO H256 to public key
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

//TODO Public key to H256
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

//TODO Signature to H512
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
	//TODO Signature from H512
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
	//TODO Public key from H256
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

/// Derive a single hard junction.
#[cfg(feature = "full_crypto")]
fn derive_hard_junction(secret_seed: &Seed, cc: &[u8; 32]) -> Seed {
	("DILITHIUM2HDKD", secret_seed, cc).using_encoded(sp_core_hashing::blake2_256)
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

	/// Derive a child key from a series of given junctions.
	fn derive<Iter: Iterator<Item = DeriveJunction>>(
		&self,
		path: Iter,
		_seed: Option<Seed>,
	) -> Result<(Pair, Option<Seed>), DeriveError> {
		let mut acc = keccak_256(self.secret.as_bytes());
		for j in path {
			match j {
				DeriveJunction::Soft(_cc) => return Err(DeriveError::SoftKeyInPath),
				DeriveJunction::Hard(cc) => acc = derive_hard_junction(&acc, &cc),
			}
		}
		Ok((Self::from_seed(&acc), Some(acc)))
	}

	/// Get the public key.
	fn public(&self) -> Public {
		let mut r = [0u8; 1312];
		r.copy_from_slice(self.public.as_bytes());
		Public(r)
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
			hex!("bf1b9b075ab1b5e7ced96af1de65896a5b5dcc78a6ae2611caab478ea437ecae")
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
				"fb34c2dc36c4f9aa2ad6105d5171e48a090d48b1a463f1dda2e0f236de7294cffe3d5870ffcc63dc0bb8f14863b871460a070847ede7df2e3cc8caa98d2be5a50413b2504f8910d89a16cd3eb69cc6156337bcf64c32e42c4c888617920b73dde9ba3e005ca3ea151c7d61af4516c3071d5a2ae2dd17e63bac84d62c7678ba3b19365836523dcbad97a8d233416dd2f730f8f2584c6b807a5f6d73c43c6c698086dba40b3440853423bd5bbe6ac118799b78eb2c525925cfb16bca76afb4478f2595f0584885c1652cee7e35289c1ea01f29d4107b61d59e7799a0035d5dc47e063d67cbabd8c482ea77327e7dba7983097da3a74a3ef32602fe4b213f23262455134c82590bfe4509a6578cc729cecc759458bb993f67444a361a53775dbe99afdae4b64494e0f6069d8f0f5cf6000b94257d834153f7ba1b3c35c4f6c401bb56e8adb7908f22748f80f400db1dad2346669b813803cbd4540e10427ae05e60fe32e6062ec7712b0dfc296ad3fdde5e5a9a1fff26a12a55c657c296a5e580bd9897da68cc933f77b5071441edaa020f5ffa8938b87e51cab43dba283794159516d0bf6b3e6922a6c35d48fd75e038a7f60f3a3b434293876d5c303c23a9833c516addbd4eacfa2f501fda256eb9b626458f23f3834b3be8e62eb962baa3ce958ebedb0d0205a7a2f9eb870d0615682851130cece158940b3e0da2a16d6dfd05cc74526b66e9f3763932a3afbfecd97e390e810e4ef33cd5842eea1de7f03a349f0945f7184b68fe92a330690e554deadfb14f77802c2656327b8234790ffc0888363d1c482e1b9bedc48e3348bd012fe26636b853421905ee0ae05681eb035c2ff6a82bccd86a6fa2e8f6a8ca8dc875a14cf4da3334fd48248947946532a565a489e9ae0c9dec97e27950df71ee6fb8ee82bf19f7066b37ef87f19bc69b6359af7fc41f8881b665f4d10fa54477b2add6b5008f58575156099cb06d0e245f37b99751207de25bb7f6044a38e092580374fb5caa994ee279864e19dafbf59f9bf79ce05f40b5292b0d912e86a7e32f2b80873c49f166f840e83bdd8be2a322c0ffc4ff5834f5cabefba6749e06d25657126a3f65e3145a7842a99f42e4f167f5d95562850a735eaa02d791065c6036b13b876a27291311f2077fead7872cc30d8f5e7d79556054c416f5185a99203edd3f20f7e38b7166d8442fd73460643752bdb1ae2b3eb171f4792948f23b744f7ce5340d24773b4bfbe967f9aaec9bd8e42ed3f5c97e6a4b600c01627c7ae67b3b1330cf20f30762fa4e0e387201b65f8859b1a481e41b9812aef9a179e9b7db1593ed0a7c5e731f7c3cb61313551d898d4a4905cd34112c12535ddca84d7d5bef653c9c945afbbcfaad460b03b3e876d54205c6e6df5f98b411cd8ddc47966627a049faf8ae1046b4d521e93a8e7eecf84be981e506e4502d6d8ef4137f609a2c8846085d7aad3e0193cc94b2e3847ed058c562eb5e095b1b4b819d93c2665a87a9f838b26acf3fe163a2ce76b55a6395d715694a95656e27c0cec2bcc1787068ebc1354ca319329277c7dc520f6db3efd3a3f0c041b749216766d1c49ba7cfd7ab482b3fe74205a85a44a8d9cdb341e7420772837c3e985be9a42bf02557af71b089b15c3a45ed1328eabd38eb4d9db324a765956dbeaebf74b04218ecefe864aa0244780d16a06b0450350d760a34e9440797fbd630f1fe04642803d30bc0249acffb02d0b100b390deda6236cbe57dd70b479ae4427e166d996143d5118dc56ab3e14fc9569a33c5b3072c94db212d34e0bb532311bbf0c36de959aca46722a546bc10e98d44eebd50fbf3652629e0"
			))
		);
		let message = b"";
		let signature = hex!("84ed909f06be255c40cb38b14f5bb723e94c69946ae95fef6541ee9f73b80b8de3bc687d266f37cd18827f5b2a4888ca53539bccb21916235fd8aec91ea410e0e3e0bf82fa12d6e1869ca2cad01dbc14c4db2ba94e402d00ede3342f050754fa318523bbd0ff604095112e795e87885c4b9709a5a19ec69cc8a97bc9cd76b4a916dbc0a9cf89ad49f35535ade3afa13122c42de9bed7e507644e23893890ab22872e53d1b24c5019f1bbc9c1fe5fc895af50c1e58f783d6f336f53899e19b95b483bb336cc7d2f6bf8fd66385cd5b446fa83cfbcb29d1e60c526702b95629328e434e531a34df6c86df17b057508804c94a3ac98ab620498ffdd655e73c588b3864a814a56ffd8a2289b327335888042bd0a0338e4d00b62d0d5a3c49d9df873abbe5241f841c4e8ca39a6fcc8c74acff09ecf89a118d518ef00a5c2e94738e482672f888ea7474f6fdc4ca2957aef6e7da4b4279c226c8166d7cdd66c45d6e6978389671e6ca39ea0d8a7fed885d67626bc2ef85d1d00d8bdc1ac76906dcb12b404f91f1fb3f1d50318a3753c100def1dea219748f2072fe6bb8244e05b2b443c660c1e070e529846cfc64edc128e6e2ade44689c7db6052647cfd348529c82a096db650f6c90f47dd5751dcaea5922482380323929a2d24edeeb83133c00bfaeb8d206d652b359c4d1b527d2436a4c538e23d2979ed5cebac5d5176074259eb3889756cf6b2cc9e29d2c418b6f8d2b3a99a4f0ef1e9de135c27f70554b1e5d14733880cb81163ba492576c69dde98b846386a5efd939c9e50e3c99ff734ceb7416055f2eb313a4db7b509839991c34cf4ca143bbf0553af99cfe36f992758ceb6442c1260981201f8f08ca60c65d403c328ce094ccce158581fd0d91d556e9d5c04b30a5f8a8cca409f9ead3e20ef1efc574529946b04a725d6b9891432da80aa801cf18609c40a8ec73dd03a83b6148b6fe7c80adfc07f5f68edd6b6f4b8a08c15a1cfe5ce0042ecb330164b69a8146c951eff6094fc44200b298638b211277ea803c861a27aac2f390271073dc1ab26c0aadf4593d4db963aeb628c986711fd601eff40e0bf74e0946ea8a586ff94715e7fa352d493cc5a0f4261f0867850c14f0bbeb64fdcd8a2c8a491054f4aaba2c084e26f021e2cdce81c062b8ddb886cab7b5cc76c0f751aaf9f08dc1810bc302211b2954b8e23e780943b2dfccd5ac6d355ae48d084b26c312e99e5e01aa0bce1d29daec14487110e9d26e904d1b1658ed1a8a5b7036ef64196688914216e60ac1d96872f8bc9c9f8f4bfb27108168e9689afae9daf46ac1b8fec2ad35cbc7977fe6fce1c0ab1585795cdfe435c97bbcda844aca4656eb2a275373cce3c27d7cf7526373017ed5f59f5ee8bf855e80fb4ee4e2b9f8bd09a2fc114c6b5b6ba48445089b94dd634a41138174d9c740a50ded0bd1b7e8f1766f59ff6ee3a233e49929b56386fcb8afc8ab405869bfa54bd2257618bca999a62d3825586bebf204fb7804c73557aa36c72aa5a637962350b354f3b353f9e1b8f4ad3bdbb7e65bca32a9d2b18e448fab5032d821e64924cd8f0d4b1184bc5c5ddbe76e0a34e5a4a5a8f47dbcd0d8c72842219d2c54717b73cd258d54f2b39526454ce522b9b9982a008fc25b29dcbd59fa1fa301ac361fb858d1a5d1d51411e8f24fd6fd5b8804b811c8fb58e5641cd6da00698d7a7166d5b04ebe39a6736e22b16cb73a562e1e261a901e825b33f3064491a0e557f8e4b67751d18ce45d713266aa0a706542d1b3d621e63cd22e0231feea51838c9c80cf031027607de782c16cd5996ecb6b52fa94ce2b995f54dcff49c5f7b45a2edaaafde9b13c9009c3e1978a5a6251ee2919870e59a4b2cf3ff313a939dd6447c74c11af6050c0220eefe79f129c290952812c05dfd05917dc2bdc18291a42a962468d2add9f753f6da5235b61ce749914a128ea4e48140178e2df2746c61afa9b30c46b8f4d47ac0359322ec837850eff99aeae6ffb87ca4f738922cefb9dfac52cc3fcffe584cb907bfaf8577db4da3b74e95c14430fc37503c222c11ce35d45cee15de96494b33ee7d9fc984d59cc1678c1274fec3416d392ac411c8ce39d7fd1134bfbbc2799701ea7c5497ed9156ed7bab71267089c0a3875f58b262440b8149008a6d24042b8843d1dfbf4d792a16180a5cc64f721c674628944147fdde67af93e78b84e8932388dcc3de29c693cf8f3428bca4c7b759287d3e6efecb0614e3153507ede66f43589637743bcda9f56bea88c466b4ce51810550eeaf3e1790918edce0d99e76cbeef6ecaad68239d8410666f7262436df5d6e7eb34e3ac1307a9a764980370c6a2d972dea364126d186a53a9702b04d695895592ee2e783f5e9fde38b30c163022786c7f67ee3b46287b11fb1e0312d3ff752cdf742065feba36599ab77efe63d6ec61e5e3c22f20dd83813a1552e616d32ddf0ba517d8c0cd4efdbc265f4be348ce71875a594531f5ef8fc972371fb8cfbdad5bf030c2c6bf54276e7a90e04fce6540b13cc42309d1907cdb020811e34ed0015911f377683381fc408c5f30f1dfc4565477c91c1a3efc8cbec9469e78f7860acef757fba27d6495e32b2ab3eb23e9f9caa8955c39c7f0475bf6291dc830618592f3adb3639a4c64f35b59d4d01bb3d098100dbab35fb83756c16b4a4a007e2c1c0c4ed6f58bbd9336be90c5472e5d7efdfc7ef167656ca9a3ea883de4f91ddbe814451ba637e363cabd30a3ea1be807ad0b650f3b2465cea99d45cd3257fcd9293845e18a40b2eed461de8ae986d54ca1192f82b95f7423226d566c3569284003c3542d366802e5aa0beccbdc1f982069b9060948aa1efb98a0c8f6048f7aa008f383f36dd8e703b14888bd43996ae01eca7e9e229c0968d1badd86eba0677cdad1f6cdc8415eb4421e9a02fd53368649a2b1d8325d26f0be743ea43d8c9ecc58cc097de0b9a7bb277d8187af0bc282e64740bf75d138d58e2226091e4dc87df07b918146af361b8ae958048d4199feaddc669957e40e0b423581fd8c1f3b379c249390a176f95e3ba2fff6f8e677fd98aa6e3458a1030629f1a6c8cebbecdec916b007cf101d4dde355fab34ca4aa49ce08c2862af811b002978c601c5a2c42a2d97e09c091ac419927783b462d2c92bc230fc4eb3cfc95705693fa3a1b421ea2aefcd81e2f505bac0ba5b42a397044eabc016dc45c51540eeb7c63e2a27659dc4c1e56b9bcc6f7b1538e761825b9329982b607972ab6a836ebb22600e109975197a10f50e1d20292a3c3d3e4461808a8b8faaabbfc4c7d6ebfc303c454a5a8c8f929aa2a7d8f7122d3b50525d5e6a79c7c9cacddbe310122021454f595e627d7e899a9dc4ced2f200000000000000000000000016233244");
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