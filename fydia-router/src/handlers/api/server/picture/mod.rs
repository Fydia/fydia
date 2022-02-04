#![allow(clippy::unwrap_used)]

use axum::{
    body::Bytes,
    extract::{Extension, Path},
    headers::HeaderName,
    response::IntoResponse,
};
use fydia_sql::{impls::server::SqlServer, sqlpool::DbConnection};
use fydia_struct::{file::File, response::FydiaResponse};
use http::{HeaderMap, HeaderValue, StatusCode};

use crate::handlers::basic::BasicValues;

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

    match BasicValues::get_user_and_server_and_check_if_joined(&headers, server_id, &database).await
    {
        Ok((_, server)) => {
            if let Ok(value) = File::get(server.icon).get_value() {
                if let Some(mimetype) = infer::get(&value) {
                    let mime_str = mimetype.to_string();
                    res.0 = StatusCode::OK;
                    res.1.insert(
                        HeaderName::from_static("Content-Type"),
                        HeaderValue::from_str(&mime_str).unwrap(),
                    );
                    res.2 = value;
                }
            }

            res
        }
        Err(error) => {
            if let Ok(error) = error.get_body() {
                res.2 = error.as_bytes().to_vec();
            } else {
                res.2 = "No error message".as_bytes().to_vec();
            }

            res
        }
    }
}

const MAX_CONTENT_LENGHT: usize = 8_000_000;

pub async fn post_picture_of_server(
    Path(server_id): Path<String>,
    body: Bytes,
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let (_, mut server) =
        match BasicValues::get_user_and_server_and_check_if_joined(&headers, server_id, &database)
            .await
        {
            Ok(v) => v,
            Err(error) => return error,
        };
    let vec_body = body.to_vec();
    if vec_body.len() > MAX_CONTENT_LENGHT {
        return FydiaResponse::new_error_custom_status("", StatusCode::PAYLOAD_TOO_LARGE);
    }
    let mimetype = if let Some(get) = infer::get(&vec_body) {
        get
    } else {
        return FydiaResponse::new_error("No body");
    };

    let mimetype_str = mimetype.extension();
    if mimetype_str != "png" && mimetype_str != "jpg" && mimetype_str != "gif" {
        return FydiaResponse::new_error_custom_status(
            "Bad Image type retry with png / jpg / gif",
            StatusCode::BAD_REQUEST,
        );
    }

    let file = File::new();
    if let Err(error) = file.create() {
        error!(error);
        return FydiaResponse::new_error_custom_status("", StatusCode::INTERNAL_SERVER_ERROR);
    };

    println!("{} / ({})", file.get_name(), vec_body.len());
    if let Err(error) = file.write(vec_body) {
        error!(error);
        return FydiaResponse::new_error_custom_status("", StatusCode::INTERNAL_SERVER_ERROR);
    };

    server.icon = file.get_name();
    if let Err(error) = server.update(&database).await {
        error!(error);
        return FydiaResponse::new_error_custom_status("", StatusCode::INTERNAL_SERVER_ERROR);
    }

    FydiaResponse::new_ok("Icon have been update")
}
