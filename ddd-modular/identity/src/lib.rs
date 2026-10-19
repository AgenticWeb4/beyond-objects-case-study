//! 【小白】Identity 限界上下文：只管「你是谁、工牌有没有过期」。
//!
//! 这里的 User **没有**密码以外的口袋：没有 karma，没有「哪天订过」。
//! 那两样分别在 Reputation、Booking。`ddd-clean` 把三样塞进同一个 User。

pub mod domain;
pub mod ports;
pub mod service;
pub mod sqlite;

pub use domain::{LoginError, RegisterError, User};
pub use restaurant_ddd_kernel::{SessionId, UserId};
pub use service::IdentityService;
pub use sqlite::SqliteIdentityStore;
