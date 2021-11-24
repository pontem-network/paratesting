use subxt::{sp_runtime, sp_core};
use sp_core::{sr25519, Pair};
use sp_core::crypto::Ss58Codec;
use sp_core::crypto::AccountId32;
use sp_runtime::traits::{Verify, IdentifyAccount};
use subxt::{Config, ExtrinsicExtraData, PairSigner};


use error::Error;
pub mod error {
    use super::sp_core;
    use sp_core::crypto::PublicError;
    use sp_core::crypto::SecretStringError;
    use sp_keyring::sr25519::ParseKeyringError;

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("{0:?}")]
        PublicError(PublicError),
        #[error("{0:?}")]
        SecretStringError(SecretStringError),
        #[error("{0:?}")]
        ParseKeyringError(ParseKeyringError),
        #[error(transparent)]
        FromHexError(#[from] hex::FromHexError),
        #[error(transparent)]
        Error(#[from] crate::BoxErr),
    }

    impl From<PublicError> for Error {
        fn from(err: PublicError) -> Self { Self::PublicError(err) }
    }

    impl From<SecretStringError> for Error {
        fn from(err: SecretStringError) -> Self { Self::SecretStringError(err) }
    }

    impl From<ParseKeyringError> for Error {
        fn from(err: ParseKeyringError) -> Self { Self::ParseKeyringError(err) }
    }
}


fn is_seed_uri<S: AsRef<str>>(s: S) -> bool { s.as_ref().starts_with("/") }
fn is_hex<S: AsRef<str>>(s: S) -> bool { s.as_ref().starts_with("0x") }


/// Interprets the string `s` in order to generate a key pair.
/// Additionally supports known keys, e.g. Alice, Bob.
///
/// See [`Pair::from_string_with_seed`][] for more extensive documentation.
///
/// [`Pair::from_string_with_seed`]: https://docs.rs/sp-core/3.0.0/sp_core/crypto/trait.Pair.html#method.from_string_with_seed
pub fn pair_from_str<P: Pair, S: AsRef<str>>(s: S) -> Result<P, Error> {
    pair_from_name(&s).or_else(move |_err| {
                          // TODO: log error as "seed isn't a name"
                          P::from_string(s.as_ref(), None).map_err(Error::from)
                      })
}

#[inline]
/// Same as [`pair_from_str`](pair_from_str) but for `sr25519::Pair`.
pub fn pair_from_str_sr25519<S: AsRef<str>>(s: S) -> Result<sr25519::Pair, Error> {
    // pair_from_str(s)
    pair_from_name_sr25519(&s).or_else(move |_err| {
                                  // TODO: log error as "seed isn't a name"
                                  Pair::from_string(s.as_ref(), None).map_err(Error::from)
                              })
}


pub fn pair_from_name<P: Pair, S: AsRef<str>>(s: S) -> Result<P, Error> {
    use sp_keyring::AccountKeyring;
    use std::str::FromStr;

    let k = AccountKeyring::from_str(&s.as_ref().to_lowercase())?;
    Ok(P::from_string(&k.to_seed(), None)?)
}

#[inline]
/// Same as [`pair_from_name`](pair_from_name) but for `sr25519::Pair`.
pub fn pair_from_name_sr25519<S: AsRef<str>>(s: S) -> Result<sr25519::Pair, Error> {
    pair_from_name(s)
}


pub fn id_from_str<S: AsRef<str>>(s: S) -> Result<AccountId32, Error> {
    // //Alice/password
    if is_seed_uri(&s) {
        let pair: sr25519::Pair = pair_from_str(s)?;
        return Ok(pair.public().into());
    }

    // 0xHEX
    if is_hex(&s) {
        // decode pub key, taking without 0x-prefix [2..]
        // TODO: return errors instead of panicking
        let bytes = hex::decode(&s.as_ref()[2..]).unwrap(); // -> FromHexError
        let pk = AccountId32::try_from(&bytes[..]).unwrap(); // -> Err(())
        return Ok(pk);
    }

    // just name:
    // TODO: mb just use use std::str::FromStr -> ParseKeyringError?
    use sp_keyring::AccountKeyring;
    if let Some(pair) = AccountKeyring::iter().find(|k| {
                                                  format!("{}", k).to_lowercase()
                                                  == s.as_ref().to_lowercase()
                                              })
    {
        return Ok(pair.to_raw_public().into());
    }


    // ss58:
    // TODO: use from_ss58check_with_version?
    let pk: AccountId32 = Ss58Codec::from_ss58check(s.as_ref())?;
    Ok(pk.into())
}


pub fn signer_from_str<T, S: AsRef<str>>(s: S) -> Result<PairSigner<T, sr25519::Pair>, Error>
    where T: Config + ExtrinsicExtraData<T>,
          // T::Signature: From<P::Signature>,
          // <T::Signature as Verify>::Signer:
          //     From<P::Public> + IdentifyAccount<AccountId = T::AccountId>,
          // P: Pair,
          T::Signature: From<sr25519::Signature>,
          <T::Signature as Verify>::Signer:
              From<sr25519::Public> + IdentifyAccount<AccountId = T::AccountId>
{
    // let res = pair_from_name(&s).or_else(|_err| /* TODO: log error as "seed isn't a name" */ pair_from_str(s));
    // let signer = PairSigner::new(res?);
    let signer = PairSigner::new(pair_from_str(s)?);
    Ok(signer)
}


#[cfg(test)]
mod tests {
    use super::*;

    const NAME: &str = "Alice";
    const SEED_URI: &str = "//Alice";
    const SEED: &str = "0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";
    const PUB_HEX: &str = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
    const SS58: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

    fn expected_pk() -> &'static [u8] {
        static mut EXPECTED: Option<Vec<u8>> = None;
        unsafe {
            if EXPECTED.is_none() {
                EXPECTED = Some(hex::decode(&PUB_HEX[2..]).unwrap());
            }
            EXPECTED.as_ref().unwrap().as_slice()
        }
    }

    #[test]
    fn id_from_hex_str() {
        let id: [u8; 32] = id_from_str(PUB_HEX).unwrap().into();
        assert_eq!(expected_pk(), id.as_ref());
    }

    #[test]
    fn id_from_ss58_str() {
        let id: [u8; 32] = id_from_str(SS58).unwrap().into();
        assert_eq!(expected_pk(), id.as_ref());
    }

    #[test]
    fn id_from_name_str() {
        let id: [u8; 32] = id_from_str(NAME).unwrap().into();
        assert_eq!(expected_pk(), id.as_ref());
    }

    #[test]
    fn id_from_seed_uri_str() {
        let id: [u8; 32] = id_from_str(SEED_URI).unwrap().into();
        assert_eq!(expected_pk(), id.as_ref());
    }

    #[test]
    #[ignore = "because currently we unable to determine the difference between the pub and private keys or seed."]
    fn id_from_seed_str() {
        let id: [u8; 32] = id_from_str(SEED).unwrap().into();
        assert_eq!(expected_pk(), id.as_ref());
    }

    #[test]
    fn pair_from_name_str() {
        let id: [u8; 32] = pair_from_name::<sr25519::Pair, _>(NAME).unwrap()
                                                                   .public()
                                                                   .into();
        assert_eq!(expected_pk(), id.as_ref());
    }

    #[test]
    fn pair_from_seed_str() {
        let id: [u8; 32] = pair_from_str::<sr25519::Pair, _>(SEED).unwrap()
                                                                  .public()
                                                                  .into();
        assert_eq!(expected_pk(), id.as_ref());
    }

    #[test]
    fn pair_from_uri_str() {
        let id: [u8; 32] = pair_from_str::<sr25519::Pair, _>(SEED_URI).unwrap()
                                                                      .public()
                                                                      .into();
        assert_eq!(expected_pk(), id.as_ref());
    }

    #[test]
    fn signer_from_uri_str() {
        let id: [u8; 32] = signer_from_str::<crate::polkadot::DefaultConfig, _>(SEED_URI).unwrap()
                                                                                         .signer()
                                                                                         .public()
                                                                                         .into();
        assert_eq!(expected_pk(), id.as_ref());
    }
}
