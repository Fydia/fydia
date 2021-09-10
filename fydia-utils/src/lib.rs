use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use rand::Rng;
use rand_core::OsRng;

pub fn generate_string(lenght: i32) -> String {
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWYZ123456789";
    let mut rng = rand::thread_rng();
    (0..lenght)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

pub fn hash(to_hash: String) -> String {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    argon2
        .hash_password_simple(to_hash.as_ref(), salt.as_ref())
        .unwrap()
        .to_string()
}

pub fn verify_password<T: Into<String>>(clear_password: String, hashed_password: T) -> bool {
    let hashed_password = hashed_password.into();
    let parsed_hash = PasswordHash::new(&hashed_password).unwrap();
    Argon2::default()
        .verify_password(clear_password.as_ref(), &parsed_hash)
        .is_ok()
}
