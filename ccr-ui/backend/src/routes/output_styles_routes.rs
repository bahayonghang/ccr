use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/output-styles",
            get(crate::api::handlers::output_styles::list_output_styles),
        )
        .route(
            "/output-styles",
            post(crate::api::handlers::output_styles::create_output_style),
        )
        .route(
            "/output-styles/{name}",
            get(crate::api::handlers::output_styles::get_output_style),
        )
        .route(
            "/output-styles/{name}",
            put(crate::api::handlers::output_styles::update_output_style),
        )
        .route(
            "/output-styles/{name}",
            delete(crate::api::handlers::output_styles::delete_output_style),
        )
}
