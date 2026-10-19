//! 【小白】Reputation 限界上下文：信誉小黑板。
//!
//! 不是 User 上的一个字段。Identity 的人不知道自己被记了几笔。

pub mod domain;
pub mod ports;
pub mod service;
pub mod sqlite;

pub use domain::STANDING_THRESHOLD;
pub use service::ReputationService;
pub use sqlite::SqliteReputationStore;
