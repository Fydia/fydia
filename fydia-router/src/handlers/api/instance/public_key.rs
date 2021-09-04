use fydia_struct::instance::RsaData;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::StatusCode;
use gotham::state::{FromState, State};

pub async fn public_key(state: State) -> HandlerResult {
    let mut res = create_response(
        &state,
        StatusCode::BAD_REQUEST,
        mime::TEXT_PLAIN_UTF_8,
        format!(""),
    );
    let rsa = RsaData::borrow_from(&state);
    if let Some(pem) = fydia_crypto::pem::key_to_string(&rsa.1) {
        res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, pem);
    }
    Ok((state, res))
}
