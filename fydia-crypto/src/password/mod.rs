use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use fydia_utils::OsRng;

pub fn hash<T: Into<String>>(to_hash: T) -> String {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    argon2
        .hash_password(to_hash.into().as_ref(), salt.as_ref())
        .unwrap()
        .to_string()
}

pub fn verify_password<T: Into<String>>(clear_password: T, hashed_password: T) -> bool {
    let hashed_password = hashed_password.into();
    let parsed_hash = PasswordHash::new(&hashed_password).unwrap();

    Argon2::default()
        .verify_password(clear_password.into().as_ref(), &parsed_hash)
        .is_ok()
}
