use axum::{
    body::Bytes,
    extract::{Extension, Path},
    headers::HeaderName,
    response::IntoResponse,
};
use fydia_sql::{
    impls::{
        server::{SqlServer, SqlServerId},
        token::SqlToken,
    },
    sqlpool::DbConnection,
};
use fydia_struct::{file::File, server::ServerId, user::Token};
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode};

pub async fn get_picture_of_server(
    Path(server_id): Path<String>,
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = (
        StatusCode::OK,
        HeaderMap::new(),
        include_bytes!("test.png").to_vec(),
    );
    res.1
        .insert(CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    if let Some(token) = Token::from_headervalue(&headers) {
        if let Some(user) = token.get_user(&database).await {
            let serverid = ServerId::new(server_id);
            if user.servers.is_join(&serverid) {
                if let Ok(server) = serverid.get_server(&database).await {
                    if let Ok(value) = File::get(server.icon).get_value() {
                        if let Some(mimetype) = infer::get(&value) {
                            let mime_str = mimetype.to_string();
                            res.0 = StatusCode::OK;
                            res.1.insert(
                                HeaderName::from_static("Content-Type"),
                                HeaderValue::from_bytes(mime_str.as_bytes()).unwrap(),
                            );
                            res.2 = value;

                            return res;
                        }
                    }
                }
            }
        }
    }

    res
}

const MAX_CONTENT_LENGHT: usize = 8_000_000;

pub async fn post_picture_of_server(
    Path(server_id): Path<String>,
    body: Bytes,
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let res = (StatusCode::OK, "");

    if let Some(token) = Token::from_headervalue(&headers) {
        if let Some(user) = token.get_user(&database).await {
            let serverid = ServerId::new(server_id);
            if user.servers.is_join(&serverid) {
                if let Ok(mut server) = serverid.get_server(&database).await {
                    let vec_body = body.to_vec();
                    if vec_body.len() > MAX_CONTENT_LENGHT {
                        return (StatusCode::PAYLOAD_TOO_LARGE, "");
                    }
                    if let Some(mimetype) = infer::get(&vec_body) {
                        let mimetype_str = mimetype.extension();
                        if mimetype_str == "png" || mimetype_str == "jpg" || mimetype_str == "gif" {
                            let file = File::new();
                            if let Err(error) = file.create() {
                                error!(error);
                                return (StatusCode::INTERNAL_SERVER_ERROR, "");
                            };

                            println!("{} / ({})", file.get_name(), vec_body.len());
                            if let Err(error) = file.write(vec_body) {
                                error!(error);
                                return (StatusCode::INTERNAL_SERVER_ERROR, "");
                            };

                            server.icon = file.get_name();
                            if let Err(error) = server.update(&database).await {
                                error!(error);
                                return (StatusCode::INTERNAL_SERVER_ERROR, "");
                            }

                            return (StatusCode::OK, "Icon have been update");
                        }

                        return (
                            StatusCode::BAD_REQUEST,
                            "Bad Image type retry with png / jpg / gif",
                        );
                    }
                }
            }
        }
    }

    res
}
