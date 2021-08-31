use std::io::{Read, Write};

use openssl::{pkey::Private, rsa::Rsa};

pub fn write(rsa: Rsa<Private>) -> std::io::Result<()> {
    let publickey = rsa.public_key_to_pem().unwrap();
    let privatekey = rsa.private_key_to_pem().unwrap();

    let create_dir = std::fs::create_dir("keys/");

    if let Err(e) = create_dir {
        return Err(e);
    }

    let publicfile = std::fs::File::create("keys/public.key");

    if let Err(e) = publicfile {
        return Err(e);
    }

    let write_publicfile = publicfile.unwrap().write(&publickey);

    if let Err(e) = write_publicfile {
        return Err(e);
    }

    let privatefile = std::fs::File::create("keys/private.key");

    if let Err(e) = privatefile {
        return Err(e);
    }

    let write_privatefile = privatefile.unwrap().write(&privatekey);

    if let Err(e) = write_privatefile {
        return Err(e);
    }

    Ok(())
}

pub fn read() -> Option<Rsa<Private>> {
    if let Ok(mut file) = std::fs::File::open("./keys/private.key") {
        let mut buf = Vec::new();
        if let Ok(_) = file.read_to_end(&mut buf) {
            if let Ok(rsa) = Rsa::private_key_from_pem(&buf) {
                return Some(rsa);
            };
            println!("Not a good key => {:?}", buf);
            return None;
        }
        println!("Whut Buffer error");
        return None;
    }
    println!("Can't read");
    return None;
}
