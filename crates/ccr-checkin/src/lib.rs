//! # ccr-checkin
//!
//! Checkin 业务域。

pub mod core;
pub mod managers;
pub mod services;

pub use ccr_db::database;

pub mod models {
    pub mod checkin {
        pub use ccr_db::models::checkin::*;
    }
}

pub use core::crypto::*;
pub use core::error::*;
pub use managers::checkin::*;
pub use models::checkin::*;
pub use services::cdk_service::{
    CdkError, CdkExtraConfig, CdkRedeemError, CdkService, CdkTopupResult,
};
pub use services::checkin_service::{CheckinExecutionResult, CheckinService, TodayCheckinStats};
