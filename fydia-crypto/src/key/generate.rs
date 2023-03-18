use openssl::{error::ErrorStack, pkey::Private, rsa::Rsa};

const LENGHT: u32 = 4096;

/// Write private key to a file
///
/// # Errors
/// Return an error if :
/// * key cannot be generated
pub fn generate() -> Result<Rsa<Private>, ErrorStack> {
    Rsa::generate(LENGHT)
}
