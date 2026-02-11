// Platform management routes
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post, put},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/platforms",
            get(crate::api::handlers::platform::list_platforms),
        )
        .route(
            "/platforms/current",
            get(crate::api::handlers::platform::get_current_platform),
        )
        .route(
            "/platforms/switch",
            post(crate::api::handlers::platform::switch_platform),
        )
        .route(
            "/platforms/capabilities",
            get(crate::api::handlers::platform::get_platform_capabilities),
        )
        .route(
            "/platforms/{name}",
            get(crate::api::handlers::platform::get_platform),
        )
        .route(
            "/platforms/{name}",
            put(crate::api::handlers::platform::update_platform),
        )
        .route(
            "/platforms/{name}/enable",
            post(crate::api::handlers::platform::enable_platform),
        )
        .route(
            "/platforms/{name}/disable",
            post(crate::api::handlers::platform::disable_platform),
        )
        .route(
            "/platforms/{name}/profile",
            get(crate::api::handlers::platform::get_platform_profile),
        )
        .route(
            "/platforms/{name}/profile",
            post(crate::api::handlers::platform::set_platform_profile),
        )
}
