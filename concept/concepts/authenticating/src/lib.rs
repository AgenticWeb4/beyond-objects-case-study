//! 【小白】工牌账：注册、登录、凭 session 认出你是谁。
//! 不管订位，不管信誉。密码明文存放，只为了 demo 能跑，别学去写生产。

pub mod actions;
pub mod adapters;
pub mod domain;
pub mod ports;

pub use actions::Authenticating;
pub use adapters::sqlite::SqliteAuthStore;
pub use domain::{LoginError, RegisterError, SessionId, UserId};
