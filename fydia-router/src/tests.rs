#[cfg(test)]
mod tests {
    use crate::*;
    use axum::{
        body::{Body, Bytes},
        http::{Request, StatusCode},
    };
    use fydia_config::DatabaseConfig;
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

        create_user(&mut app).await?;
        create_user_without_email(&mut app).await?;
        login_user(&mut app).await?;
        login_user_with_bad_json(&mut app).await?;

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

    async fn login_user(app: &mut Router) -> Result<(), ()> {
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
                    .body(Body::from(
                        r#"{"bad":"", "json":""}"#,
                    ))
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
}
