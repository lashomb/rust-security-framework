//! Encryption and Decryption transform support.

use core_foundation::base::TCFType;
use core_foundation::data::CFData;
use core_foundation::error::CFError;
use core_foundation::string::CFString;
use core_foundation_sys::data::CFDataRef;
use core_foundation_sys::string::CFStringRef;
use security_framework_sys::encrypt_transform::*;
use security_framework_sys::transform::*;
use std::ptr;

use key::SecKey;
use os::macos::transform::SecTransform;

#[derive(Debug, Copy, Clone)]
/// The padding scheme to use for encryption.
pub struct Padding(CFStringRef);

impl Padding {
    /// Do not pad.
    pub fn none() -> Padding {
        unsafe { Padding(kSecPaddingNoneKey) }
    }

    /// Use PKCS#1 padding.
    pub fn pkcs1() -> Padding {
        unsafe { Padding(kSecPaddingPKCS1Key) }
    }

    /// Use PKCS#5 padding.
    pub fn pkcs5() -> Padding {
        unsafe { Padding(kSecPaddingPKCS5Key) }
    }

    /// Use PKCS#7 padding.
    pub fn pkcs7() -> Padding {
        unsafe { Padding(kSecPaddingPKCS7Key) }
    }

    /// Use OAEP padding.
    pub fn oaep() -> Padding {
        unsafe { Padding(kSecPaddingOAEPKey) }
    }

    fn to_str(&self) -> CFString {
        unsafe { CFString::wrap_under_get_rule(self.0) }
    }
}

/// The cipher mode to use.
///
/// Only applies to AES encryption.
#[derive(Debug, Copy, Clone)]
pub struct Mode(CFStringRef);

#[allow(missing_docs)]
impl Mode {
    pub fn none() -> Mode {
        unsafe { Mode(kSecModeNoneKey) }
    }

    pub fn ecb() -> Mode {
        unsafe { Mode(kSecModeECBKey) }
    }

    pub fn cbc() -> Mode {
        unsafe { Mode(kSecModeCBCKey) }
    }

    pub fn cfb() -> Mode {
        unsafe { Mode(kSecModeCFBKey) }
    }

    pub fn ofb() -> Mode {
        unsafe { Mode(kSecModeOFBKey) }
    }

    fn to_str(&self) -> CFString {
        unsafe { CFString::wrap_under_get_rule(self.0) }
    }
}

/// A builder for encryption and decryption transform operations.
#[derive(Default)]
pub struct Builder {
    padding: Option<Padding>,
    mode: Option<Mode>,
    iv: Option<CFData>,
}

impl Builder {
    /// Creates a new `Builder` with a default configuration.
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Selects the padding scheme to use.
    ///
    /// If not set, an appropriate scheme will be selected for you.
    pub fn padding(&mut self, padding: Padding) -> &mut Builder {
        self.padding = Some(padding);
        self
    }

    /// Selects the encryption mode to use.
    ///
    /// If not set, an appropriate mode will be selected for you.
    pub fn mode(&mut self, mode: Mode) -> &mut Builder {
        self.mode = Some(mode);
        self
    }

    /// Sets the initialization vector to use.
    ///
    /// If not set, an appropriate value will be supplied for you.
    pub fn iv(&mut self, iv: CFData) -> &mut Builder {
        self.iv = Some(iv);
        self
    }

    /// Encrypts data with a provided key.
    pub fn encrypt(&self, key: &SecKey, data: &CFData) -> Result<CFData, CFError> {
        unsafe {
            let mut error = ptr::null_mut();
            let transform = SecEncryptTransformCreate(key.as_concrete_TypeRef(), &mut error);
            if transform.is_null() {
                return Err(CFError::wrap_under_create_rule(error));
            }
            let transform = SecTransform::wrap_under_create_rule(transform);

            self.finish(transform, data)
        }
    }

    /// Decrypts data with a provided key.
    pub fn decrypt(&self, key: &SecKey, data: &CFData) -> Result<CFData, CFError> {
        unsafe {
            let mut error = ptr::null_mut();
            let transform = SecDecryptTransformCreate(key.as_concrete_TypeRef(), &mut error);
            if transform.is_null() {
                return Err(CFError::wrap_under_create_rule(error));
            }
            let transform = SecTransform::wrap_under_create_rule(transform);

            self.finish(transform, data)
        }
    }

    fn finish(&self, mut transform: SecTransform, data: &CFData) -> Result<CFData, CFError> {
        unsafe {
            if let Some(ref padding) = self.padding {
                let key = CFString::wrap_under_get_rule(kSecPaddingKey);
                transform.set_attribute(&key, &padding.to_str())?;
            }

            if let Some(ref mode) = self.mode {
                let key = CFString::wrap_under_get_rule(kSecEncryptionMode);
                transform.set_attribute(&key, &mode.to_str())?;
            }

            if let Some(ref iv) = self.iv {
                let key = CFString::wrap_under_get_rule(kSecIVKey);
                transform.set_attribute(&key, iv)?;
            }

            let key = CFString::wrap_under_get_rule(kSecTransformInputAttributeName);
            transform.set_attribute(&key, data)?;

            let result = transform.execute()?;
            Ok(CFData::wrap_under_get_rule(
                result.as_CFTypeRef() as CFDataRef
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use core_foundation::data::CFData;
    use hex::FromHex;

    use super::*;
    use key::SecKey;
    use os::macos::item::KeyType;
    use os::macos::key::SecKeyExt;

    #[test]
    fn cbc_mmt_256() {
        // test 9
        let key = "87725bd43a45608814180773f0e7ab95a3c859d83a2130e884190e44d14c6996";
        let iv = "e49651988ebbb72eb8bb80bb9abbca34";
        let ciphertext = "5b97a9d423f4b97413f388d9a341e727bb339f8e18a3fac2f2fb85abdc8f135deb30054a\
                          1afdc9b6ed7da16c55eba6b0d4d10c74e1d9a7cf8edfaeaa684ac0bd9f9d24ba674955c7\
                          9dc6be32aee1c260b558ff07e3a4d49d24162011ff254db8be078e8ad07e648e6bf56793\
                          76cb4321a5ef01afe6ad8816fcc7634669c8c4389295c9241e45fff39f3225f7745032da\
                          eebe99d4b19bcb215d1bfdb36eda2c24";
        let plaintext = "bfe5c6354b7a3ff3e192e05775b9b75807de12e38a626b8bf0e12d5fff78e4f1775aa7d79\
                         2d885162e66d88930f9c3b2cdf8654f56972504803190386270f0aa43645db187af41fcea\
                         639b1f8026ccdd0c23e0de37094a8b941ecb7602998a4b2604e69fc04219585d854600e0a\
                         d6f99a53b2504043c08b1c3e214d17cde053cbdf91daa999ed5b47c37983ba3ee254bc5c7\
                         93837daaa8c85cfc12f7f54f699f";

        let key = Vec::<u8>::from_hex(key).unwrap();
        let key = CFData::from_buffer(&key);
        let key = SecKey::from_data(KeyType::aes(), &key).unwrap();

        let iv = Vec::<u8>::from_hex(iv).unwrap();

        let ciphertext = Vec::<u8>::from_hex(ciphertext).unwrap();

        let plaintext = Vec::<u8>::from_hex(plaintext).unwrap();

        let decrypted = Builder::new()
            .padding(Padding::none())
            .iv(CFData::from_buffer(&iv))
            .decrypt(&key, &CFData::from_buffer(&ciphertext))
            .unwrap();

        assert_eq!(plaintext, decrypted.bytes());

        let encrypted = Builder::new()
            .padding(Padding::none())
            .iv(CFData::from_buffer(&iv))
            .encrypt(&key, &CFData::from_buffer(&plaintext))
            .unwrap();

        assert_eq!(ciphertext, encrypted.bytes());
    }
}
