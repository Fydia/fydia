#![allow(clippy::unwrap_used)]

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

use crate::handlers::basic::BasicValues;

pub async fn get_picture_of_server(
    Path(server_id): Path<String>,
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult {
    let (_, server) =
        BasicValues::get_user_and_server_and_check_if_joined(&headers, server_id, &database)
            .await?;

    let value = File::get(server.icon)
        .get_value()
        .map_err(|_| FydiaResponse::new_error("Cannot get file"))?;

    let mime_str = infer::get(&value)
        .ok_or_else(|| FydiaResponse::new_error("Cannot get the mimetype"))?
        .to_string();

    let mut result = FydiaResponse::new_bytes_ok(value);
    result.add_headers("Content-Type", &mime_str);

    return Ok(result);
}

const MAX_CONTENT_LENGHT: usize = 8_000_000;

pub async fn post_picture_of_server(
    Path(server_id): Path<String>,
    body: Bytes,
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult {
    let (_, mut server) =
        BasicValues::get_user_and_server_and_check_if_joined(&headers, server_id, &database)
            .await?;
    let vec_body = body.to_vec();

    if vec_body.len() > MAX_CONTENT_LENGHT {
        return Err(FydiaResponse::new_error_custom_status(
            "",
            StatusCode::PAYLOAD_TOO_LARGE,
        ));
    }

    let mimetype = infer::get(&vec_body).ok_or_else(|| {
        return FydiaResponse::new_error("No body");
    })?;

    let mimetype_str = mimetype.extension();
    if mimetype_str != "png" && mimetype_str != "jpg" && mimetype_str != "gif" {
        return Err(FydiaResponse::new_error_custom_status(
            "Bad Image type retry with png / jpg / gif",
            StatusCode::BAD_REQUEST,
        ));
    }

    let file = File::new();
    let name = file.get_name();
    let len_of_boy = vec_body.len();
    println!("{name} / ({len_of_boy})");

    file.create_and_write(vec_body).map_err(|error| {
        error!(error);
        return FydiaResponse::new_error_custom_status("", StatusCode::INTERNAL_SERVER_ERROR);
    })?;

    server.icon = file.get_name();
    server
        .update(&database)
        .await
        .map(|_| FydiaResponse::new_ok("Icon have been update"))
        .map_err(|error| {
            error!(error);
            FydiaResponse::new_error_custom_status("", StatusCode::INTERNAL_SERVER_ERROR)
        })
}
