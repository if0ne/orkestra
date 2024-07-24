use tracing::info;

use crate::{
    models::session::Session,
    shared::{context::Context, services::sesser::Sesser},
};

pub fn filter_sessions<S: Sesser>(context: Context<S>, code: Option<String>) -> Vec<Session> {
    if let Some(code) = code {
        info!(
            target: "filter_sessions",
            event = "Filter sessions by code",
            code = code
        );

        context.sesser().filter_by_code(code)
    } else {
        info!(
            target: "filter_sessions",
            event = "Fetching all sessions",
        );

        context.sesser().get_all_sessions()
    }
}
