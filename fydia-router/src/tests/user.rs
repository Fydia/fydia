/* mod user_test {
    use crate::{get_axum_router, user_routes};
    use axum::body::Body;
    use axum::body::HttpBody;
    use http::{Method, Request, Response, StatusCode};
    use tower::ServiceExt;
    use fydia_config::Config;

    #[tokio::test]
    pub async fn create_user() {
        let app = get_axum_router(Config::);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/create")
                    .method(Method::POST)
                    .body(Body::from("".to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();

        println!(
            "{:?}",
            String::from_utf8(
                hyper::body::to_bytes(response.into_body())
                    .await
                    .unwrap()
                    .to_vec()
            )
        );
        assert_eq!(status, StatusCode::OK);
    }
}
*/
