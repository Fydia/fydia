#[cfg(test)]
mod tests {
    use crate::*;
    use axum::http::StatusCode;
    use futures::StreamExt;
    use fydia_config::DatabaseConfig;
    use http::{header::CONTENT_TYPE, HeaderValue};
    use serde_json::Value;
    use std::net::SocketAddr;
    use time::Instant;
    use tokio::task::JoinHandle;

    pub fn get_sqlite() -> Config {
        Config {
            instance: fydia_config::InstanceConfig {
                domain: "".to_string(),
            },
            server: fydia_config::ServerConfig {
                ip: "0.0.0.0".to_string(),
                port: 4000,
            },
            database: DatabaseConfig::new(
                "fydia_test",
                0,
                "",
                "",
                "fydia_test",
                fydia_config::DatabaseType::Sqlite,
            ),
        }
    }

    pub async fn get_router() -> Router {
        let config = get_sqlite();
        let db = super::super::get_database_connection(&config.database).await;
        super::super::get_axum_router(
            db,
            &config.instance,
            &config.format_ip(),
            *&config.server.port as u16,
        )
        .await
    }

    const IP: &str = "127.0.0.1:8000";
    const IP_WITH_HTTP: &str = "http://127.0.0.1:8000";
    const IP_WITH_WS: &str = "ws://127.0.0.1:8000";

    #[tokio::test]
    async fn test() -> Result<(), String> {
        let listener = std::net::TcpListener::bind(IP.parse::<SocketAddr>().unwrap()).unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(get_router().await.into_make_service())
                .await
                .unwrap();
        });

        let mut token = String::new();
        let mut server_id = String::new();
        let mut channel_id = String::new();

        create_user().await?;
        create_user_without_email().await?;
        login_user(&mut token).await?;
        login_user_with_bad_json().await?;
        token_verify(&token).await?;
        get_me(&token).await?;
        create_a_server(&token, &mut server_id).await?;
        get_server_info(&token, &server_id).await?;
        post_server_picture(&token, &server_id).await?;
        get_server_picture(&token, &server_id).await?;
        create_channel(&token, &server_id, &mut channel_id).await?;
        post_messages(&token, &server_id, &channel_id).await?;
        start_typing(&token, &server_id, &channel_id).await?;
        stop_typing(&token, &server_id, &channel_id).await?;
        update_name_channel(&token, &server_id, &channel_id).await?;
        update_description_channel(&token, &server_id, &channel_id).await?;
        test_wb_post_message(&token, &server_id, &channel_id).await?;
        // TODO: ReAdd When TypingManager will work nicely
        //test_wb_start_typing(&token, &server_id, &channel_id).await?;
        //test_wb_stop_typing(&token, &server_id, &channel_id).await?;
        Ok(())
    }
    async fn create_user() -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{}/api/user/create", IP_WITH_HTTP))
            .body(r#"{"name":"default", "email":"default@default", "password":"default"}"#)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::OK {
            if body == r#"{"status":"OK","content":"Register successfully"}"#.to_string() {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn create_user_without_email() -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{}/api/user/create", IP_WITH_HTTP))
            .body(r#"{"name":"default", "password":"default"}"#)
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::BAD_REQUEST {
            if body == r#"{"status":"Error","content":"No `email` in JSON"}"#.to_string() {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn login_user(token: &mut String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{}/api/user/login", IP_WITH_HTTP))
            .body(r#"{"email":"default@default", "password":"default"}"#)
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::OK {
            if body.starts_with(r#"{"status":"OK","content":""#) {
                *token = body
                    .strip_prefix(r#"{"status":"OK","content":""#)
                    .unwrap()
                    .strip_suffix("\"}")
                    .unwrap()
                    .to_string();
                return Ok(());
            }
        }

        Err(body)
    }

    async fn login_user_with_bad_json() -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{}/api/user/login", IP_WITH_HTTP))
            .body(r#"{"bad":"", "json":""}"#)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::BAD_REQUEST {
            if body.starts_with(r#"{"status":"Error","content":""#) {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn token_verify(token: &String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .get(format!("{}/api/user/token/verify", IP_WITH_HTTP))
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::OK {
            return Ok(());
        }

        Err(body)
    }

    async fn get_me(token: &String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .get(format!("{}/api/user/me", IP_WITH_HTTP))
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        if status == StatusCode::OK {
            if serde_json::from_str::<Value>(&body)
                .unwrap()
                .get("content")
                .is_some()
            {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn create_a_server(token: &String, server_id: &mut String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{}/api/server/create", IP_WITH_HTTP))
            .body(r#"{"name":"default_name_server"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::OK {
            if let Some(content) = serde_json::from_str::<Value>(&body).unwrap().get("content") {
                if let Some(token_str) = content.as_str() {
                    *server_id = token_str.to_string();
                    return Ok(());
                }
            }
        }

        Err(body)
    }

    async fn get_server_info(token: &String, server_id: &String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .get(format!("{}/api/server/{}", IP_WITH_HTTP, server_id))
            .body(r#"{"name":"default_name_server"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::OK {
            if serde_json::from_str::<Value>(&body)
                .unwrap()
                .get("content")
                .is_some()
            {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn get_server_picture(token: &String, server_id: &String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .get(format!("{}/api/server/{}/picture", IP_WITH_HTTP, server_id))
            .body(r#"{"name":"default_name_server"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.bytes().await.unwrap();

        if status == StatusCode::OK {
            if body.to_vec() == include_bytes!("image.png").to_vec() {
                return Ok(());
            }
        }

        Err(String::from("No Message on get_server_picture"))
    }

    async fn post_server_picture(token: &String, server_id: &String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{}/api/server/{}/picture", IP_WITH_HTTP, server_id))
            .body(include_bytes!("image.png").to_vec())
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::OK {
            if serde_json::from_str::<Value>(&body)
                .unwrap()
                .get("content")
                .is_some()
            {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn create_channel(
        token: &String,
        server_id: &String,
        channel_id: &mut String,
    ) -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!(
                "{}/api/server/{}/channel/create",
                IP_WITH_HTTP, server_id
            ))
            .body(r#"{"name": "channel_default", "type":"TEXT"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();
        println!("{}", body);

        if status == StatusCode::OK {
            if let Some(content) = serde_json::from_str::<Value>(&body).unwrap().get("content") {
                *channel_id = content.as_str().unwrap().to_string();
                return Ok(());
            }
        }

        Err(body)
    }

    async fn post_messages(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!(
                "{}/api/server/{}/channel/{}/messages",
                IP_WITH_HTTP, server_id, channel_id
            ))
            .body(r#"{"content": "MESSAGE", "type":"TEXT"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .header(
                CONTENT_TYPE,
                HeaderValue::from_bytes(b"application/json").unwrap(),
            )
            .send()
            .await
            .unwrap();
        let statuscode = response.status();
        let body = response.text().await.unwrap();

        if statuscode == StatusCode::OK {
            if serde_json::from_str::<Value>(&body)
                .unwrap()
                .get("content")
                .is_some()
            {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn start_typing(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!(
                "{}/api/server/{}/channel/{}/typing/start",
                IP_WITH_HTTP, server_id, channel_id
            ))
            .body(r#"{"content": "MESSAGE", "type":"TEXT"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .header(
                CONTENT_TYPE,
                HeaderValue::from_bytes(b"application/json").unwrap(),
            )
            .send()
            .await
            .unwrap();
        let statuscode = response.status();
        let body = response.text().await.unwrap();

        if statuscode == StatusCode::OK {
            return Ok(());
        }

        Err(body)
    }
    async fn stop_typing(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!(
                "{}/api/server/{}/channel/{}/typing/stop",
                IP_WITH_HTTP, server_id, channel_id
            ))
            .body(r#"{"content": "MESSAGE", "type":"TEXT"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .header(
                CONTENT_TYPE,
                HeaderValue::from_bytes(b"application/json").unwrap(),
            )
            .send()
            .await
            .unwrap();
        let statuscode = response.status();
        let body = response.text().await.unwrap();

        if statuscode == StatusCode::OK {
            return Ok(());
        }

        Err(body)
    }

    async fn update_name_channel(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        let response = reqwest::Client::new()
            .put(format!(
                "{}/api/server/{}/channel/{}/name",
                IP_WITH_HTTP, server_id, channel_id
            ))
            .body(r#"{"name":"new_name"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .header(
                CONTENT_TYPE,
                HeaderValue::from_bytes(b"application/json").unwrap(),
            )
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        if status == StatusCode::OK {
            if serde_json::from_str::<Value>(&body)
                .unwrap()
                .get("content")
                .is_some()
            {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn update_description_channel(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        let response = reqwest::Client::new()
            .put(format!(
                "{}/api/server/{}/channel/{}/description",
                IP_WITH_HTTP, server_id, channel_id
            ))
            .body(r#"{"description":"new_name"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .header(
                CONTENT_TYPE,
                HeaderValue::from_bytes(b"application/json").unwrap(),
            )
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();
        if status == StatusCode::OK {
            if serde_json::from_str::<Value>(&body)
                .unwrap()
                .get("content")
                .is_some()
            {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn test_wb_post_message(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        //ws://127.0.0.1:8080/api/user/websocket?token=default_token
        let url =
            url::Url::parse(format!("{}/api/user/websocket?token={}", IP_WITH_WS, token).as_str())
                .unwrap();
        let a: JoinHandle<Result<(), String>> = tokio::spawn(async move {
            let (mut socket, _) = tokio_tungstenite::connect_async(url)
                .await
                .expect("Connection error");
            let time = Instant::now();
            while let Some(Ok(wb)) = socket.next().await {
                match wb {
                    tokio_tungstenite::tungstenite::Message::Text(e) => {
                        println!("{}", e);
                        return Ok(());
                    }
                    _ => {
                        if time.elapsed().whole_seconds() == 10 {
                            break;
                        }
                    }
                };
            }

            Err(String::from("No message"))
        });

        post_messages(token, server_id, channel_id).await?;
        return a.await.unwrap();
    }

    async fn _test_wb_start_typing(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        //ws://127.0.0.1:8080/api/user/websocket?token=default_token
        let url =
            url::Url::parse(format!("{}/api/user/websocket?token={}", IP_WITH_WS, token).as_str())
                .unwrap();
        let a: JoinHandle<Result<(), String>> = tokio::spawn(async move {
            let (mut socket, _) = tokio_tungstenite::connect_async(url)
                .await
                .expect("Connection error");
            let time = Instant::now();
            while let Some(Ok(wb)) = socket.next().await {
                match wb {
                    tokio_tungstenite::tungstenite::Message::Text(e) => {
                        println!("{}", e);
                        return Ok(());
                    }
                    _ => {
                        if time.elapsed().whole_seconds() == 10 {
                            break;
                        }
                    }
                };
            }

            Err(String::from("No message"))
        });
        start_typing(token, server_id, channel_id).await?;
        return a.await.unwrap();
    }

    async fn _test_wb_stop_typing(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        //ws://127.0.0.1:8080/api/user/websocket?token=default_token
        let url =
            url::Url::parse(format!("{}/api/user/websocket?token={}", IP_WITH_WS, token).as_str())
                .unwrap();
        let a: JoinHandle<Result<(), String>> = tokio::spawn(async move {
            let (mut socket, _) = tokio_tungstenite::connect_async(url)
                .await
                .expect("Connection error");
            let time = Instant::now();
            while let Some(Ok(wb)) = socket.next().await {
                match wb {
                    tokio_tungstenite::tungstenite::Message::Text(e) => {
                        println!("{}", e);
                        return Ok(());
                    }
                    _ => {
                        if time.elapsed().whole_seconds() == 10 {
                            break;
                        }
                    }
                };
            }

            Err(String::from("No message"))
        });
        std::thread::sleep(Duration::from_secs(2));
        stop_typing(token, server_id, channel_id).await?;
        return a.await.unwrap();
    }
}
