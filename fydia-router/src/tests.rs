#[cfg(test)]
mod tests {
    use crate::*;
    use axum::{
        body::{Body, Bytes},
        http::{Request, StatusCode},
    };
    use fydia_config::DatabaseConfig;
    use http::{HeaderValue, header::CONTENT_TYPE};
    use serde_json::Value;
    use tower::ServiceExt;

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

    pub fn println_body_as_string(body: &Bytes) {
        println!("{}", String::from_utf8(body.to_vec()).unwrap());
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

    #[tokio::test]
    async fn test() -> Result<(), ()> {
        let mut app = get_router().await;
        let mut token = String::new();
        let mut server_id = String::new();
        let mut channel_id = String::new();

        create_user(&mut app).await?;
        create_user_without_email(&mut app).await?;
        login_user(&mut app, &mut token).await?;
        login_user_with_bad_json(&mut app).await?;
        token_verify(&mut app, &token).await?;
        get_me(&mut app, &token).await?;
        create_a_server(&mut app, &token, &mut server_id).await?;
        get_server_info(&mut app, &token, &server_id).await?;
        post_server_picture(&mut app, &token, &server_id).await?;
        get_server_picture(&mut app, &token, &server_id).await?;
        create_channel(&mut app, &token, &server_id, &mut channel_id).await?;
        post_messages(&mut app, &token, &server_id, &channel_id).await?;
        update_name_channel(&mut app,&token, &server_id, &channel_id).await?;
        update_description_channel(&mut app, &token, &server_id, &channel_id).await?;

        Ok(())
    }
    async fn create_user(app: &mut Router) -> Result<(), ()> {
        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/user/create")
                    .body(Body::from(
                        r#"{"name":"default", "email":"default@default", "password":"default"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            if &body[..] == br#"{"status":"OK","content":"Register successfully"}"# {
                return Ok(());
            }
        }

        Err(())
    }

    async fn create_user_without_email(app: &mut Router) -> Result<(), ()> {
        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/user/create")
                    .body(Body::from(r#"{"name":"default", "password":"default"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::BAD_REQUEST {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            println_body_as_string(&body);
            if &body[..] == br#"{"status":"Error","content":"Json error"}"# {
                return Ok(());
            }
        }

        Err(())
    }

    async fn login_user(app: &mut Router, token: &mut String) -> Result<(), ()> {
        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/user/login")
                    .body(Body::from(
                        r#"{"email":"default@default", "password":"default"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);
            if body_str.starts_with(r#"{"status":"OK","content":""#) {
                println!(
                    "{}",
                    body_str
                        .strip_prefix(r#"{"status":"OK","content":""#)
                        .unwrap()
                );
                *token = body_str
                    .strip_prefix(r#"{"status":"OK","content":""#)
                    .unwrap()
                    .strip_suffix("\"}")
                    .unwrap()
                    .to_string();
                return Ok(());
            }
        }

        Err(())
    }

    async fn login_user_with_bad_json(app: &mut Router) -> Result<(), ()> {
        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/user/login")
                    .body(Body::from(r#"{"bad":"", "json":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::BAD_REQUEST {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);
            if body_str.starts_with(r#"{"status":"Error","content":""#) {
                return Ok(());
            }
        }

        Err(())
    }

    async fn token_verify(app: &mut Router, token: &String) -> Result<(), ()> {
        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let mut req = Request::builder()
            .method("GET")
            .uri("/api/user/token/verify");

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        let response = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();

        if response.status() == StatusCode::OK {
            return Ok(());
        }

        Err(())
    }

    async fn get_me(app: &mut Router, token: &String) -> Result<(), ()> {
        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let mut req = Request::builder().method("GET").uri("/api/user/me");

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        let response = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();

        if response.status() == StatusCode::OK {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);
            if serde_json::from_str::<Value>(&body_str)
                .unwrap()
                .get("content")
                .is_some()
            {
                return Ok(());
            }
        }

        Err(())
    }

    async fn create_a_server(
        app: &mut Router,
        token: &String,
        server_id: &mut String,
    ) -> Result<(), ()> {
        let mut req = Request::builder().method("GET").uri("/api/server/create");

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        let response = app
            .oneshot(
                req.body(Body::from(r#"{"name":"default_name_server"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);
            if let Some(content) = serde_json::from_str::<Value>(&body_str)
                .unwrap()
                .get("content")
            {
                if let Some(token_str) = content.as_str() {
                    *server_id = token_str.to_string();
                    return Ok(());
                }
            }
        }

        Err(())
    }

    async fn get_server_info(
        app: &mut Router,
        token: &String,
        server_id: &String,
    ) -> Result<(), ()> {
        let mut req = Request::builder()
            .method("GET")
            .uri(format!("/api/server/{}", server_id));

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        let response = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();

        if response.status() == StatusCode::OK {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);
            if serde_json::from_str::<Value>(&body_str)
                .unwrap()
                .get("content")
                .is_some()
            {
                return Ok(());
            }
        }

        Err(())
    }

    async fn get_server_picture(
        app: &mut Router,
        token: &String,
        server_id: &String,
    ) -> Result<(), ()> {
        let mut req = Request::builder()
            .method("GET")
            .uri(format!("/api/server/{}/picture", server_id));

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        let response = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();

        if response.status() == StatusCode::OK {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            if body.to_vec() == include_bytes!("image.png").to_vec() {
                return Ok(());
            }
        }

        Err(())
    }

    async fn post_server_picture(
        app: &mut Router,
        token: &String,
        server_id: &String,
    ) -> Result<(), ()> {
        let mut req = Request::builder()
            .method("POST")
            .uri(format!("/api/server/{}/picture", server_id));

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        let response = app
            .oneshot(
                req.body(Body::from(include_bytes!("image.png").to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);
            if serde_json::from_str::<Value>(&body_str)
                .unwrap()
                .get("content")
                .is_some()
            {
                return Ok(());
            }
        }

        Err(())
    }

    async fn create_channel(
        app: &mut Router,
        token: &String,
        server_id: &String,
        channel_id: &mut String,
    ) -> Result<(), ()> {
        let mut req = Request::builder()
            .method("GET")
            .uri(format!("/api/server/{}/channel/create", server_id));

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        let response = app
            .oneshot(
                req.body(Body::from(r#"{"name": "channel_default", "type":"TEXT"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);
            if let Some(content)=  serde_json::from_str::<Value>(&body_str)
                .unwrap()
                .get("content")
            {
                *channel_id = content.as_str().unwrap().to_string();
                return Ok(());
            }
        }

        Err(())
    }

    async fn post_messages(
        app: &mut Router,
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), ()> {
        let mut req = Request::builder()
            .method("POST")
            .uri(format!("/api/server/{}/channel/{}/messages", server_id, channel_id));

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        req.headers_mut().unwrap().insert(
            CONTENT_TYPE,
            HeaderValue::from_bytes(b"application/json").unwrap(),
        );

        let response = app
            .oneshot(
                req.body(Body::from(r#"{"content":"MESSAGE", "type": "TEXT"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);
            if serde_json::from_str::<Value>(&body_str)
                .unwrap()
                .get("content").is_some()
            {
                return Ok(());
            }
        }

        Err(())
    }

    async fn update_name_channel(
        app: &mut Router,
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), ()> {
        let mut req = Request::builder()
            .method("PUT")
            .uri(format!("/api/server/{}/channel/{}/name", server_id, channel_id));

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        req.headers_mut().unwrap().insert(
            CONTENT_TYPE,
            HeaderValue::from_bytes(b"application/json").unwrap(),
        );

        let response = app
            .oneshot(
                req.body(Body::from(r#"{"name":"new_name"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
            let statuscode = response.status();

            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);

        if statuscode == StatusCode::OK {

            if serde_json::from_str::<Value>(&body_str)
                .unwrap()
                .get("content").is_some()
            {
                return Ok(());
            }
        }

        Err(())
    }

    async fn update_description_channel(
        app: &mut Router,
        token: &String,
        server_id: &String,
        channel_id: &String,
    ) -> Result<(), ()> {
        let mut req = Request::builder()
            .method("PUT")
            .uri(format!("/api/server/{}/channel/{}/description", server_id, channel_id));

        req.headers_mut().unwrap().insert(
            fydia_struct::user::HEADERNAME,
            HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        req.headers_mut().unwrap().insert(
            CONTENT_TYPE,
            HeaderValue::from_bytes(b"application/json").unwrap(),
        );

        let response = app
            .oneshot(
                req.body(Body::from(r#"{"description":"new_name"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
            let statuscode = response.status();

            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println_body_as_string(&body);

        if statuscode == StatusCode::OK {

            if serde_json::from_str::<Value>(&body_str)
                .unwrap()
                .get("content").is_some()
            {
                return Ok(());
            }
        }

        Err(())
    }
}
