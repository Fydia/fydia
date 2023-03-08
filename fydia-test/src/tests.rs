use crate::TestContext;

const CONTEXT: &'static TestContext = &TestContext::new("127.0.0.1", 8080);

const TOKEN: &'static str = "default_token";
const SERVER: &'static str = "server_default_id";
const CHANNEL: &'static str = "channel_default";

#[tokio::test]
pub async fn create_account() -> Result<(), String> {
    CONTEXT
        .post("/api/user/create")
        .body(r#"{"name":"default","email":"default@default", "password":"default"}"#)
        .expect_statuscode(200)
        .expect_body(r#"{"status":"Ok","content":"Register successfully"}"#)
        .send()
        .await
}

#[tokio::test]
pub async fn login() -> Result<(), String> {
    CONTEXT
        .post("/api/user/login")
        .body(r#"{"email":"default@default", "password":"default"}"#)
        .expect_statuscode(200)
        .send()
        .await
}

#[tokio::test]
pub async fn token_verify_ok() -> Result<(), String> {
    CONTEXT
        .get("/api/user/token/verify")
        .header("Authorization", TOKEN)
        .expect_statuscode(200)
        .send()
        .await
}

#[tokio::test]
pub async fn token_verify_bad() -> Result<(), String> {
    CONTEXT
        .get("/api/user/token/verify")
        .header("Authorization", "THISIANUNVALIDTOKEN")
        .expect_statuscode(400)
        .send()
        .await
}

#[tokio::test]
pub async fn get_me() -> Result<(), String> {
    CONTEXT
        .get("/api/user/me")
        .header("Authorization", TOKEN)
        .expect_statuscode(200)
        .send()
        .await
}

#[tokio::test]
pub async fn get_me_bad_token() -> Result<(), String> {
    CONTEXT
        .get("/api/user/me")
        .header("Authorization", "THISIANUNVALIDTOKEN")
        .expect_statuscode(400)
        .send()
        .await
}

#[tokio::test]
pub async fn create_server() -> Result<(), String> {
    CONTEXT
        .post("/api/server/create")
        .body(r#"{"name":"default_name_server"}"#)
        .header("Authorization", TOKEN)
        .expect_statuscode(200)
        .send()
        .await
}

#[tokio::test]
pub async fn create_server_empty_name() -> Result<(), String> {
    CONTEXT
        .post("/api/server/create")
        .body(r#"{"name":""}"#)
        .header("Authorization", TOKEN)
        .expect_statuscode(400)
        .send()
        .await
}

#[tokio::test]
pub async fn get_server_info() -> Result<(), String> {
    CONTEXT
        .get(format!("/api/server/{SERVER}").as_str())
        .header("Authorization", TOKEN)
        .expect_statuscode(200)
        .send()
        .await
}

#[tokio::test]
pub async fn get_server_info_with_wrong_serverid() -> Result<(), String> {
    CONTEXT
        .get("/api/server/WRONGSERVERID")
        .header("Authorization", TOKEN)
        .expect_statuscode(400)
        .send()
        .await
}

#[tokio::test]
pub async fn create_a_channel() -> Result<(), String> {
    CONTEXT
        .post(format!("/api/server/{SERVER}/channel/create").as_str())
        .body(r#"{"name":"channel_default", "type":"TEXT"}"#)
        .header("Authorization", TOKEN)
        .expect_statuscode(200)
        .send()
        .await
}

#[tokio::test]
pub async fn create_a_channel_without_name() -> Result<(), String> {
    CONTEXT
        .post(format!("/api/server/{SERVER}/channel/create").as_str())
        .body(r#"{"name":"", "type":"TEXT"}"#)
        .header("Authorization", TOKEN)
        .expect_statuscode(400)
        .send()
        .await
}

#[tokio::test]
pub async fn create_a_channel_without_token() -> Result<(), String> {
    CONTEXT
        .post(format!("/api/server/{SERVER}/channel/create").as_str())
        .body(r#"{"name":"channel_default", "type":"TEXT"}"#)
        .expect_statuscode(400)
        .send()
        .await
}

#[tokio::test]
pub async fn start_typing() -> Result<(), String> {
    CONTEXT
        .post(format!("/api/server/{SERVER}/channel/{CHANNEL}/typing/start").as_str())
        .header("Authorization", TOKEN)
        .expect_statuscode(200)
        .send()
        .await
}

#[tokio::test]
pub async fn stop_typing() -> Result<(), String> {
    CONTEXT
        .post(format!("/api/server/{SERVER}/channel/{CHANNEL}/typing/stop").as_str())
        .header("Authorization", TOKEN)
        .expect_statuscode(200)
        .send()
        .await
}
