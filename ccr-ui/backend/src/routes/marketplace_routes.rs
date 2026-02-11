// Marketplace Routes
// 资源市场 API 路由

use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::api::handlers::marketplace;

pub fn routes() -> Router<AppState> {
    Router::new()
        // 获取所有市场项目
        .route("/marketplace", get(marketplace::list_marketplace_items))
        // 按分类获取市场项目
        .route(
            "/marketplace/category/{category}",
            get(marketplace::list_items_by_category),
        )
        // 获取已安装的项目
        .route(
            "/marketplace/installed",
            get(marketplace::list_installed_items),
        )
        // 安装市场项目
        .route("/marketplace/install", post(marketplace::install_item))
        // 卸载市场项目
        .route(
            "/marketplace/uninstall/{item_id}",
            delete(marketplace::uninstall_item),
        )
}
