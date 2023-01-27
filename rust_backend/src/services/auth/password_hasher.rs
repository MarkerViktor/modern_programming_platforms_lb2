use ring::{digest::SHA256_OUTPUT_LEN, pbkdf2};
use std::num::NonZeroU32;

#[derive(Clone)]
pub struct PasswordHasher {
    pub salt: String,
    pub iterations: NonZeroU32,
}

impl PasswordHasher {
    pub fn from_env() -> Self {
        let salt = std::env::var("PASSWORD_HASH_SALT").expect("PASSWORD_HASH_SALT must be set.");
        let iterations = NonZeroU32::new(100_000).unwrap();
        PasswordHasher { salt, iterations }
    }

    pub fn get_hash(&self, password: &str) -> [u8; 32] {
        let mut hash = [0u8; SHA256_OUTPUT_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            self.iterations,
            self.salt.as_bytes(),
            password.as_bytes(),
            &mut hash,
        );
        hash
    }
}
