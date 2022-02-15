use std::io::{Read, Write};

use openssl::{pkey::Private, rsa::Rsa};

pub fn write(rsa: Rsa<Private>) -> std::io::Result<()> {
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

pub fn read() -> Option<Rsa<Private>> {
    let mut file = std::fs::File::open("./keys/private.key").ok()?;
    let mut buf = Vec::new();

    file.read_to_end(&mut buf).ok()?;

    Rsa::private_key_from_pem(&buf).ok()
}
