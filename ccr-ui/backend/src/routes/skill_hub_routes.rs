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
            "/skill_hub/marketplace/refresh-cache",
            post(crate::api::handlers::skill_hub::refresh_marketplace_cache),
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
        // === 新增端点: 多源安装 ===
        .route(
            "/skill_hub/import/github",
            post(crate::api::handlers::skill_hub::import_github),
        )
        .route(
            "/skill_hub/import/local",
            post(crate::api::handlers::skill_hub::import_local),
        )
        .route(
            "/skill_hub/import/npx",
            post(crate::api::handlers::skill_hub::import_npx),
        )
        .route(
            "/skill_hub/batch-install",
            post(crate::api::handlers::skill_hub::batch_install),
        )
        .route(
            "/skill_hub/npx/status",
            get(crate::api::handlers::skill_hub::npx_status),
        )
        .route(
            "/skill_hub/browse-folder",
            post(crate::api::handlers::skill_hub::browse_folder),
        )
}
