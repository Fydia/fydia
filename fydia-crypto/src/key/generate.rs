use openssl::{error::ErrorStack, pkey::Private, rsa::Rsa};

const LENGHT: u32 = 4096;

pub fn generate_key() -> Result<Rsa<Private>, ErrorStack> {
    Rsa::generate(LENGHT)
}
