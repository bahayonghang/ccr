use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/skill_hub/marketplace/trending",
            get(crate::api::handlers::skill_hub::marketplace_trending),
        )
        .route(
            "/skill_hub/marketplace/search",
            get(crate::api::handlers::skill_hub::marketplace_search),
        )
        .route(
            "/skill_hub/agents",
            get(crate::api::handlers::skill_hub::list_agents),
        )
        .route(
            "/skill_hub/agents/{agent}/skills",
            get(crate::api::handlers::skill_hub::list_agent_skills),
        )
        .route(
            "/skill_hub/install",
            post(crate::api::handlers::skill_hub::install),
        )
        .route(
            "/skill_hub/remove",
            post(crate::api::handlers::skill_hub::remove),
        )
        // Unified skills endpoints
        .route(
            "/skill_hub/unified",
            get(crate::api::handlers::skill_hub::list_unified_skills),
        )
        .route(
            "/skill_hub/unified/{platform}",
            get(crate::api::handlers::skill_hub::list_unified_skills_by_platform),
        )
        // Skill content read/write
        .route(
            "/skill_hub/skill/content",
            get(crate::api::handlers::skill_hub::get_skill_content)
                .post(crate::api::handlers::skill_hub::save_skill_content),
        )
}
