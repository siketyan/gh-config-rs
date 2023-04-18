#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Windows(#[from] windows::Error),

    #[error(transparent)]
    Macos(#[from] macos::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub trait GhKeyring {
    fn get(&self, host: &str) -> Result<Vec<u8>>;
}

mod windows {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {}
}

mod macos {
    use super::*;
    use base64::Engine;
    use security_framework::os::macos::keychain::{SecKeychain, SecPreferencesDomain};

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Security.framework returned an error: {0}")]
        SecurityFramework(#[from] security_framework::base::Error),

        #[error("The encoded token is invalid as a hex string: {0}")]
        InvalidHex(#[from] hex::FromHexError),

        #[error("The encoded token is invalid as a Base64 string: {0}")]
        InvalidBase64(#[from] base64::DecodeError),

        #[error("The token is formatted in an unknown encoding.")]
        UnknownFormat,
    }

    type Result<T> = std::result::Result<T, Error>;

    enum GhEncodedToken {
        Hex(Vec<u8>),
        Base64(Vec<u8>),
    }

    impl TryFrom<&[u8]> for GhEncodedToken {
        type Error = Error;

        fn try_from(value: &[u8]) -> Result<Self> {
            if let Some(s) = value.strip_prefix(b"go-keyring-encoded:") {
                return Ok(Self::Hex(s.to_vec()));
            }

            if let Some(s) = value.strip_prefix(b"go-keyring-base64:") {
                return Ok(Self::Base64(s.to_vec()));
            }

            Err(Error::UnknownFormat)
        }
    }

    impl GhEncodedToken {
        fn decode(&self) -> Result<Vec<u8>> {
            Ok(match self {
                Self::Hex(s) => hex::decode(s)?,
                Self::Base64(s) => base64::engine::general_purpose::STANDARD.decode(s)?,
            })
        }
    }

    pub struct Keychain;

    impl GhKeyring for Keychain {
        fn get(&self, host: &str) -> super::Result<Vec<u8>> {
            (|| {
                Ok(GhEncodedToken::try_from(
                    SecKeychain::default_for_domain(SecPreferencesDomain::User)?
                        .find_generic_password(format!("gh:{}", host).as_str(), "")?
                        .0
                        .as_ref(),
                )?
                .decode()?)
            })()
            .map_err(super::Error::Macos)
        }
    }
}

mod linux {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {}
}

#[cfg(target_os = "macos")]
pub use macos::Keychain as Keyring;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_from_keyring() {
        assert!(!Keyring.get("github.com").unwrap().is_empty());
    }
}
