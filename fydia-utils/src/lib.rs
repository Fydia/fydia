use rand::Rng;

pub use rand_core::OsRng;

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
