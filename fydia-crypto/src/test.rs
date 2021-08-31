mod test {

    #[test]
    pub fn encrypt_decrypt_aes() {
        use crate::decrypt::aes_decrypt;
        use crate::encrypt::aes_encrypt;
        use crate::pem::get_key_from_string;
        use fydia_utils::generate_string;
        let message = generate_string(4000);
        let key = crate::key::generate::generate_key().unwrap();
        let public = key.public_key_to_pem().unwrap();
        let public_key = get_key_from_string(String::from_utf8(public).unwrap()).unwrap();
        let encrypt = aes_encrypt(public_key, message.clone()).unwrap();
        let decrypt = aes_decrypt(key, encrypt).unwrap();

        assert_eq!(message, decrypt)
    }
}
