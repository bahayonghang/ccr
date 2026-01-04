// Skills management routes
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router {
    Router::new()
        .route("/skills", get(crate::api::handlers::skills::list_skills))
        .route("/skills", post(crate::api::handlers::skills::add_skill))
        .route(
            "/skills/{name}",
            get(crate::api::handlers::skills::get_skill),
        )
        .route(
            "/skills/{name}",
            put(crate::api::handlers::skills::update_skill),
        )
        .route(
            "/skills/{name}",
            delete(crate::api::handlers::skills::delete_skill),
        )
        .route(
            "/skills/repositories",
            get(crate::api::handlers::skills::list_repositories),
        )
        .route(
            "/skills/repositories",
            post(crate::api::handlers::skills::add_repository),
        )
        .route(
            "/skills/repositories/{name}",
            delete(crate::api::handlers::skills::remove_repository),
        )
        .route(
            "/skills/repositories/{name}/scan",
            get(crate::api::handlers::skills::scan_repository),
        )
}
