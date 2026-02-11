// Agent management routes
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/agents", get(crate::api::handlers::agents::list_agents))
        .route("/agents", post(crate::api::handlers::agents::add_agent))
        .route(
            "/agents/{name}",
            get(crate::api::handlers::agents::get_agent),
        )
        .route(
            "/agents/{name}",
            put(crate::api::handlers::agents::update_agent),
        )
        .route(
            "/agents/{name}",
            delete(crate::api::handlers::agents::delete_agent),
        )
        .route(
            "/agents/{name}/toggle",
            put(crate::api::handlers::agents::toggle_agent),
        )
}
