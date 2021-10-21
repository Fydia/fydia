use fydia_struct::response::FydiaResponse;
use gotham::{handler::HandlerResult, helpers::http::response::create_empty_response, state::State};
use reqwest::StatusCode;

pub async fn delete_direct_message(state: State) -> HandlerResult {
    let mut res = create_empty_response(&state, StatusCode::NOT_IMPLEMENTED);
    FydiaResponse::new_error_custom_status("", StatusCode::NOT_IMPLEMENTED).update_response(&mut res);
    Ok((state, res))
}