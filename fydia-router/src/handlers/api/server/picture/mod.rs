#![allow(clippy::unwrap_used)]

use std::str::FromStr;

use axum::extract::{Extension, Path, State};
use fydia_sql::{impls::server::SqlServer, sqlpool::DbConnection};
use fydia_struct::{
    file::File,
    response::{FydiaResponse, FydiaResult, IntoFydia},
};
use fydia_utils::http::{HeaderMap, StatusCode};
use mime::Mime;

use crate::{handlers::basic::BasicValues, ServerState};

/// Return icon of server
///
/// # Errors
/// Return an error if serverid isn't valid
pub async fn get_picture_of_server(
    Path(server_id): Path<String>,
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult {
    let (_, server) =
        BasicValues::get_user_and_server_and_check_if_joined(&headers, &server_id, &database)
            .await?;

    let value = File::get(server.icon).get_value().map_err(|error| {
        error!("{error}");
        "Cannot get file".into_error()
    })?;

    let mime_str = infer::get(&value)
        .ok_or_else(|| "Cannot get the mimetype".into_error())?
        .to_string();

    let mime = Mime::from_str(mime_str.as_str()).map_err(|error| {
        error!("{error}");
        "Cannot convert mime".into_error()
    })?;

    Ok(FydiaResponse::BytesWithContentType(value, mime))
}

const MAX_CONTENT_LENGHT: usize = 8_000_000;

/// Change server picture
///
/// # Errors
/// This function will return an error if file given isn't a file or if file is too large
/// of if server doesn't exist
pub async fn post_picture_of_server(
    State(state): State<ServerState>,
    Path(server_id): Path<String>,
    headers: HeaderMap,
) -> FydiaResult {
    let (_, mut server) =
        BasicValues::get_user_and_server_and_check_if_joined(&headers, &server_id, &state.database)
            .await?;
    let body = vec![];
    if body.len() > MAX_CONTENT_LENGHT {
        return Err("".into_error_with_statuscode(StatusCode::PAYLOAD_TOO_LARGE));
    }

    let mimetype = infer::get(&body).ok_or_else(|| "No body".into_error())?;

    let mimetype_str = mimetype.extension();
    if mimetype_str != "png" && mimetype_str != "jpg" && mimetype_str != "gif" {
        return Err("Bad Image type retry with png / jpg / gif".into_error());
    }

    let file = File::new();

    file.create_and_write(&body).map_err(|error| {
        error!("{error}");
        "".into_server_error()
    })?;

    server.icon = file.get_name();

    server
        .update(&state.database)
        .await
        .map(|_| "Icon have been update".into_ok())
}
