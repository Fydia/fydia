#![allow(clippy::unwrap_used)]

use std::str::FromStr;

use axum::{
    body::Bytes,
    extract::{Extension, Path},
};
use fydia_sql::{impls::server::SqlServer, sqlpool::DbConnection};
use fydia_struct::{
    file::File,
    response::{FydiaResponse, FydiaResult},
};
use http::{HeaderMap, StatusCode};
use mime::Mime;

use crate::handlers::basic::BasicValues;

pub async fn get_picture_of_server<'a>(
    Path(server_id): Path<String>,
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let (_, server) =
        BasicValues::get_user_and_server_and_check_if_joined(&headers, server_id, &database)
            .await?;

    let value = File::get(server.icon)
        .get_value()
        .map_err(|_| FydiaResponse::TextError("Cannot get file"))?;

    let mime_str = infer::get(&value)
        .ok_or(FydiaResponse::TextError("Cannot get the mimetype"))?
        .to_string();

    let mime = Mime::from_str(mime_str.as_str())
        .map_err(|_| FydiaResponse::TextError("Cannot convert mime"))?;

    Ok(FydiaResponse::BytesWithContentType(value, mime))
}

const MAX_CONTENT_LENGHT: usize = 8_000_000;

pub async fn post_picture_of_server<'a>(
    Path(server_id): Path<String>,
    body: Bytes,
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let (_, mut server) =
        BasicValues::get_user_and_server_and_check_if_joined(&headers, server_id, &database)
            .await?;

    if body.len() > MAX_CONTENT_LENGHT {
        return Err(FydiaResponse::TextErrorWithStatusCode(
            StatusCode::PAYLOAD_TOO_LARGE,
            "",
        ));
    }

    let mimetype = infer::get(&body).ok_or(FydiaResponse::TextError("No body"))?;

    let mimetype_str = mimetype.extension();
    if mimetype_str != "png" && mimetype_str != "jpg" && mimetype_str != "gif" {
        return Err(FydiaResponse::TextErrorWithStatusCode(
            StatusCode::BAD_REQUEST,
            "Bad Image type retry with png / jpg / gif",
        ));
    }

    let file = File::new();

    file.create_and_write(&body).map_err(|error| {
        error!(error);
        FydiaResponse::TextErrorWithStatusCode(StatusCode::INTERNAL_SERVER_ERROR, "")
    })?;

    server.icon = file.get_name();

    server
        .update(&database)
        .await
        .map(|_| FydiaResponse::Text("Icon have been update"))
        .map_err(|error| {
            error!(error);
            FydiaResponse::TextErrorWithStatusCode(StatusCode::INTERNAL_SERVER_ERROR, "")
        })
}
