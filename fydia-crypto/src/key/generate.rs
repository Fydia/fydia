use openssl::{error::ErrorStack, pkey::Private, rsa::Rsa};

pub fn generate_key() -> Result<Rsa<Private>, ErrorStack> {
    Rsa::generate(2048)
}
