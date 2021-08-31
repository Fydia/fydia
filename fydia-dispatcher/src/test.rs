#[cfg(test)]
mod tests {

    use fydia_struct::channel::ChannelId;
    use fydia_struct::messages::{Message, MessageType, SqlDate};
    use fydia_struct::user::User;

    #[tokio::test]
    pub async fn get_keys() {
        let reqwest = reqwest::get("http://127.0.0.1:8080/api/instance/public_key").await;

        if let Ok(res) = reqwest {
            if let Ok(text) = res.text().await {
                println!("{}", text);
                assert!(true);
                return;
            }
            assert!(false);
        }
        assert!(false);
    }

    #[tokio::test]
    pub async fn encrypt_message() {
        let reqwest = reqwest::get("http://127.0.0.1:8080/api/instance/public_key").await;

        if let Ok(res) = reqwest {
            if let Ok(text) = res.text().await {
                if let Some(public) = fydia_crypto::pem::get_key_from_string(text) {
                    let message = Message::new(
                        String::from("This is a message"),
                        MessageType::TEXT,
                        false,
                        SqlDate::now(),
                        User::default(),
                        ChannelId::new(String::new()),
                    );

                    let string = serde_json::to_string(&message).unwrap();

                    let _ = fydia_crypto::encrypt::encrypt(public, string);

                    println!("Encrypted Successfully");

                    assert!(true);
                    return;
                };
            }
        }
        assert!(false);
    }

    /*#[tokio::test]
    pub async fn send_dispatch() {
        let server = tokio::spawn(async {
            std::process::Command::new("cargo")
                .args(&["run", "-p", "fydia"])
                .spawn()
                .unwrap()
                .wait()
                .unwrap()
        });
        let message = generate_string(4000);
        let reqwest = reqwest::get("http://127.0.0.1:8080/api/instance/public_key").await;
        if let Ok(res) = reqwest {
            if let Ok(text) = res.text().await {
                if let Some(public) = fydia_crypto::pem::get_key_from_string(text) {
                    let key = fydia_crypto::key::generate::generate_key().unwrap();
                    let encrypt = aes_encrypt(public, message.clone()).unwrap();
                }
            }
        }
    }*/
}
