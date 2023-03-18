use std::io::Write;

use openssl::{pkey::Private, rsa::Rsa};

/// Write private key to a file
///
/// # Errors
/// Return an if :
/// * File cannot be written
/// * File cannot be created
pub fn write(rsa: &Rsa<Private>) -> std::io::Result<()> {
    match (rsa.public_key_to_pem(), rsa.private_key_to_pem()) {
        (Ok(publickey), Ok(privatekey)) => {
            std::fs::create_dir("keys/")?;

            let mut publickeyfile = std::fs::File::create("keys/public.key")?;

            publickeyfile.write_all(&publickey)?;

            let mut privatekeyfile = std::fs::File::create("keys/private.key")?;

            privatekeyfile.write_all(&privatekey)?;

            Ok(())
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Public key to pem error",
        )),
    }
}

/// Return Private key
///
/// # Errors
/// Return an if :
/// * File cannot be read
/// * File cannot be converted as RSA key
#[must_use]
pub fn read() -> Option<Rsa<Private>> {
    let buf = std::fs::read("./keys/private.key").ok()?;

    Rsa::private_key_from_pem(&buf).ok()
}
