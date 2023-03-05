use reqwest::Method;

use crate::{create_result, create_test};

const TOKEN: &'static str = "default_token";
const SERVER: &'static str = "server_default_id";
const CHANNEL: &'static str = "channel_default";

#[tokio::test]
pub async fn create_account() {
    let mut test = create_test!(
        Method::POST,
        "/api/user/create",
        r#"{"name":"default","email":"default@default", "password":"default"}"#
    );

    create_result!(
        200,
        r#"{"status":"Ok","content":"Register successfully"}"#,
        test
    );

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn login() {
    let mut test = create_test!(
        Method::POST,
        "/api/user/login",
        r#"{"email":"default@default", "password":"default"}"#
    );

    create_result!(200, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn token_verify_ok() {
    let mut test = create_test!(
        Method::GET,
        "/api/user/token/verify",
        r#""#,
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(200, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn token_verify_bad() {
    let mut test = create_test!(
        Method::GET,
        "/api/user/token/verify",
        r#""#,
        vec![(
            "Authorization".to_string(),
            "THISIANUNVALIDTOKEN".to_string()
        )]
    );

    create_result!(400, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn get_me() {
    let mut test = create_test!(
        Method::GET,
        "/api/user/me",
        r#""#,
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(200, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn get_me_bad_token() {
    let mut test = create_test!(
        Method::GET,
        "/api/user/token/verify",
        r#""#,
        vec![(
            "Authorization".to_string(),
            "THISIANUNVALIDTOKEN".to_string()
        )]
    );

    create_result!(400, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn create_server() {
    let mut test = create_test!(
        Method::POST,
        "/api/server/create",
        r#"{"name":"default_name_server"}"#,
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(200, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn create_server_empty_name() {
    let mut test = create_test!(
        Method::POST,
        "/api/server/create",
        r#"{"name":""}"#,
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(400, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn get_server_info() {
    let mut test = create_test!(
        Method::GET,
        format!("/api/server/{SERVER}"),
        r#""#.to_string(),
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(200, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn get_server_info_with_wrong_serverid() {
    let mut test = create_test!(
        Method::GET,
        "/api/server/WRONGSERVERID",
        r#""#,
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(400, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn create_a_channel() {
    let mut test = create_test!(
        Method::POST,
        format!("/api/server/{SERVER}/channel/create"),
        r#"{"name":"channel_default", "type":"TEXT"}"#.to_string(),
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(200, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn create_a_channel_without_name() {
    let mut test = create_test!(
        Method::POST,
        format!("/api/server/{SERVER}/channel/create"),
        r#"{"name":"", "type":"TEXT"}"#.to_string(),
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(400, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn create_a_channel_without_token() {
    let mut test = create_test!(
        Method::POST,
        format!("/api/server/{SERVER}/channel/create"),
        r#"{"name":"channel_default", "type":"TEXT"}"#.to_string()
    );

    create_result!(400, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn start_typing() {
    let mut test = create_test!(
        Method::POST,
        format!("/api/server/{SERVER}/channel/{CHANNEL}/typing/start"),
        r#""#.to_string(),
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(200, r#""#.to_string(), test);

    test.send().await.unwrap();
}

#[tokio::test]
pub async fn stop_typing() {
    let mut test = create_test!(
        Method::POST,
        format!("/api/server/{SERVER}/channel/{CHANNEL}/typing/start"),
        r#""#.to_string(),
        vec![("Authorization".to_string(), TOKEN.to_string())]
    );

    create_result!(200, r#""#.to_string(), test);

    test.send().await.unwrap();
}
