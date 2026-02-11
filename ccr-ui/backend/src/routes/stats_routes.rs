// Statistics routes
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/stats/cost",
            get(crate::api::handlers::stats::cost_overview),
        )
        .route(
            "/stats/cost/today",
            get(crate::api::handlers::stats::cost_today),
        )
        .route(
            "/stats/cost/week",
            get(crate::api::handlers::stats::cost_week),
        )
        .route(
            "/stats/cost/month",
            get(crate::api::handlers::stats::cost_month),
        )
        .route(
            "/stats/cost/trend",
            get(crate::api::handlers::stats::cost_trend),
        )
        .route(
            "/stats/cost/by-model",
            get(crate::api::handlers::stats::cost_by_model),
        )
        .route(
            "/stats/cost/by-project",
            get(crate::api::handlers::stats::cost_by_project),
        )
        .route(
            "/stats/provider-usage",
            get(crate::api::handlers::stats::provider_usage),
        )
        .route(
            "/stats/cost/top-sessions",
            get(crate::api::handlers::stats::cost_top_sessions),
        )
        .route(
            "/stats/summary",
            get(crate::api::handlers::stats::stats_summary),
        )
        .route(
            "/stats/heatmap",
            get(crate::api::handlers::stats::get_heatmap_data),
        )
}
