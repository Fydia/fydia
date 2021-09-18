use gotham::state::State;

pub mod api;
pub mod event;
pub mod federation;

pub fn default(state: State) -> (State, String) {
    let e = "Default. This request will be implemented soon".to_string();
    (state, e)
}
