#[cfg(test)]
mod tests {
    use crate::*;
    use axum::http::StatusCode;
    use futures::StreamExt;
    use fydia_config::DatabaseConfig;
    use fydia_struct::{
        event::{Event, EventContent},
        server::ServerId,
    };
    use http::{header::CONTENT_TYPE, HeaderValue};
    use serde_json::Value;
    use std::net::SocketAddr;
    use time::Instant;
    use tokio::{net::TcpStream, task::JoinHandle};
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

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

    pub async fn get_websocket(token: &str) -> Websocket {
        let url =
            url::Url::parse(format!("{IP_WITH_WS}/api/user/websocket?token={token}").as_str())
                .unwrap();

        if let Ok(map) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|error| {
                println!("{}", error);
                error
            })
        {
            return map.0;
        }

        panic!("Error")
    }

    pub async fn get_router() -> Router {
        let config = get_sqlite();
        let db = super::super::get_database_connection(&config.database)
            .await
            .unwrap();
        super::super::get_axum_router(
            db,
            &config.instance,
            &config.format_ip(),
            *&config.server.port as u16,
        )
        .await
        .unwrap()
    }

    type Websocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

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
        create_empty_user().await?;
        login_user(&mut token).await?;
        login_user_with_bad_json().await?;
        login_empty_user().await?;
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
        test_delete_message(&token, &server_id, &channel_id).await?;
        test_update_message(&token, &server_id, &channel_id).await?;
        // TODO: ReAdd When TypingManager will work nicely
        //test_wb_start_typing(&token, &server_id, &channel_id).await?;
        //test_wb_stop_typing(&token, &server_id, &channel_id).await?;
        Ok(())
    }

    async fn create_user() -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{IP_WITH_HTTP}/api/user/create"))
            .body(r#"{"name":"default", "email":"default@default", "password":"default"}"#)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::OK {
            if body == r#"{"status":"Ok","content":"Register successfully"}"#.to_string() {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn create_user_without_email() -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{IP_WITH_HTTP}/api/user/create"))
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

    async fn create_empty_user() -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{IP_WITH_HTTP}/api/user/create"))
            .body(r#"{"name":"", "email":"", "password":""}"#)
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

    async fn login_user(token: &mut String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{IP_WITH_HTTP}/api/user/login"))
            .body(r#"{"email":"default@default", "password":"default"}"#)
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::OK {
            if body.starts_with(r#"{"status":"Ok","content":""#) {
                *token = body
                    .strip_prefix(r#"{"status":"Ok","content":""#)
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
            .post(format!("{IP_WITH_HTTP}/api/user/login"))
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

    async fn login_empty_user() -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{IP_WITH_HTTP}/api/user/login"))
            .body(r#"{"email":"", "password":""}"#)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();

        if status == StatusCode::BAD_REQUEST {
            if body.starts_with(r#"{"status":"Error","#) {
                return Ok(());
            }
        }

        Err(body)
    }

    async fn token_verify(token: &String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .get(format!("{IP_WITH_HTTP}/api/user/token/verify"))
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
            .get(format!("{IP_WITH_HTTP}/api/user/me"))
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
            .post(format!("{IP_WITH_HTTP}/api/server/create"))
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
            .get(format!("{IP_WITH_HTTP}/api/server/{server_id}"))
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
            .get(format!("{IP_WITH_HTTP}/api/server/{server_id}/picture"))
            .body(r#"{"name":"default_name_server"}"#)
            .header(
                fydia_struct::user::HEADERNAME,
                HeaderValue::from_bytes(token.as_bytes()).unwrap(),
            )
            .send()
            .await
            .unwrap();

        let status = response.status();

        if status == StatusCode::OK {
            let body = response.bytes().await.unwrap().to_vec();
            let image = include_bytes!("image.png").to_vec();
            if image.is_empty() {
                return Err("Image is empty".to_string());
            }
            for (n, i) in body.iter().enumerate() {
                if let Some(e) = image.get(n) {
                    if e != i {
                        println!("At {n} : {e} != {i}");
                        return Err("Not good image".to_string());
                    }
                }
            }

            return Ok(());
        } else {
            Err(response.text().await.unwrap())
        }
    }

    async fn post_server_picture(token: &String, server_id: &String) -> Result<(), String> {
        let response = reqwest::Client::new()
            .post(format!("{IP_WITH_HTTP}/api/server/{server_id}/picture"))
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
                "{IP_WITH_HTTP}/api/server/{server_id}/channel/create"
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
                "{IP_WITH_HTTP}/api/server/{server_id}/channel/{channel_id}/messages",
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
                "{IP_WITH_HTTP}/api/server/{server_id}/channel/{channel_id}/typing/start"
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
                "{IP_WITH_HTTP}/api/server/{server_id}/channel/{channel_id}/typing/stop"
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
                "{IP_WITH_HTTP}/api/server/{server_id}/channel/{channel_id}/name",
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
                "{IP_WITH_HTTP}/api/server/{server_id}/channel/{channel_id}/description",
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
        let mut socket = get_websocket(token).await;

        let a: JoinHandle<Result<(), String>> = tokio::spawn(async move {
            let time = Instant::now();
            loop {
                if let Some(Ok(wb)) = socket.next().await {
                    if let tokio_tungstenite::tungstenite::Message::Text(_) = wb {
                        return Ok(());
                    }
                }

                if time.elapsed().whole_seconds() > 10 {
                    panic!("No message");
                }
            }
        });

        post_messages(token, server_id, channel_id).await?;

        return a.await.unwrap();
    }

    async fn test_delete_message(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        let mut socket = get_websocket(token).await;
        let (sender, receiver) = flume::unbounded::<Event>();
        let a: JoinHandle<Result<(), String>> = tokio::spawn(async move {
            let mut i = 0;
            loop {
                let message = socket.next().await;
                if let Some(Ok(wb)) = message {
                    i += 1;
                    if let tokio_tungstenite::tungstenite::Message::Text(string) = wb {
                        let event = serde_json::from_str::<Event>(&string).unwrap();
                        sender.send_async(event).await.unwrap();
                    }

                    if i == 2 {
                        break;
                    }
                }
            }

            Ok(())
        });

        post_messages(token, server_id, channel_id).await?;

        loop {
            let message = receiver.recv().unwrap();

            let messageid = if let EventContent::Message { content } = message.content {
                content.id
            } else {
                return Err(String::from("Bad Type of EventContent"));
            };

            reqwest::Client::new()
                    .delete(format!(
                        "{IP_WITH_HTTP}/api/server/{server_id}/channel/{channel_id}/messages/{messageid}",
                    ))
                    .header(
                        fydia_struct::user::HEADERNAME,
                        HeaderValue::from_bytes(token.as_bytes()).unwrap(),
                    )
                    .send()
                    .await
                    .unwrap();

            let ev = receiver.recv_async().await.unwrap();
            if ev
                == Event::new(
                    ServerId::new(server_id),
                    EventContent::MessageDelete {
                        message_id: messageid,
                    },
                )
            {
                a.abort();
                return Ok(());
            }
            break;
        }

        return a.await.unwrap();
    }

    async fn test_update_message(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        let mut socket = get_websocket(token).await;
        let (sender, receiver) = flume::unbounded::<Event>();
        let a: JoinHandle<Result<(), String>> = tokio::spawn(async move {
            let mut i = 0;
            loop {
                let message = socket.next().await;
                if let Some(Ok(wb)) = message {
                    i += 1;
                    if let tokio_tungstenite::tungstenite::Message::Text(string) = wb {
                        let event = serde_json::from_str::<Event>(&string).unwrap();
                        sender.send_async(event).await.unwrap();
                    }

                    if i == 2 {
                        break;
                    }
                }
            }

            Ok(())
        });

        post_messages(token, server_id, channel_id).await?;

        loop {
            let message = receiver.recv().unwrap();

            let messageid = if let EventContent::Message { content } = message.content {
                content.id
            } else {
                return Err(String::from("Bad Type of EventContent"));
            };

            reqwest::Client::new()
                    .post(format!(
                        "{IP_WITH_HTTP}/api/server/{server_id}/channel/{channel_id}/messages/{messageid}",
                    ))
                    .header(
                        fydia_struct::user::HEADERNAME,
                        HeaderValue::from_bytes(token.as_bytes()).unwrap(),
                    )
                    .body(r#"{"content": "MESSAGE", "type":"TEXT"}"#)
                    .send()
                    .await
                    .unwrap();

            let ev = receiver.recv_async().await.unwrap();
            if let EventContent::MessageUpdate { .. } = ev.content {
                a.abort();
                return Ok(());
            }
            break;
        }

        return a.await.unwrap();
    }

    async fn _test_wb_start_typing(
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), String> {
        //ws://127.0.0.1:8080/api/user/websocket?token=default_token
        let url =
            url::Url::parse(format!("{IP_WITH_WS}/api/user/websocket?token={token}").as_str())
                .unwrap();
        let a: JoinHandle<Result<(), String>> = tokio::spawn(async move {
            let (mut socket, _) = tokio_tungstenite::connect_async(url)
                .await
                .expect("Connection error");
            let time = Instant::now();
            while let Some(Ok(wb)) = socket.next().await {
                match wb {
                    tokio_tungstenite::tungstenite::Message::Text(_) => {
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
            url::Url::parse(format!("{IP_WITH_WS}/api/user/websocket?token={token}").as_str())
                .unwrap();
        let a: JoinHandle<Result<(), String>> = tokio::spawn(async move {
            let (mut socket, _) = tokio_tungstenite::connect_async(url)
                .await
                .expect("Connection error");
            let time = Instant::now();
            while let Some(Ok(wb)) = socket.next().await {
                match wb {
                    tokio_tungstenite::tungstenite::Message::Text(_) => {
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
