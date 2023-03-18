use std::borrow::Cow;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use fydia_utils::OsRng;

/// Hash `T` with argon2
///
/// # Errors
/// Return an error if :
/// * `T` value cannot be hashed
pub fn hash<T: Into<String>>(to_hash: T) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    Ok(argon2
        .hash_password(to_hash.into().as_ref(), &salt)
        .map_err(|error| error.to_string())?
        .to_string())
}

#[must_use]
pub fn verify<'a>(clear_password: Cow<'a, str>, hashed_password: Cow<'a, str>) -> bool {
    let hashed_password = hashed_password;
    let clear_password = clear_password;

    let password = PasswordHash::new(&hashed_password);

    if let Ok(hash) = password {
        Argon2::default()
            .verify_password(clear_password.as_bytes(), &hash)
            .is_ok()
    } else {
        false
    }
}
