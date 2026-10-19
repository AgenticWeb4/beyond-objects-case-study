//! 【小白】订座本。只回答一件事：这份「到时候给你留位」的承诺还算不算数。
//!
//! 不管登录密码，不管门口还挂不挂空位牌，不管放鸽子扣不扣分。
//! 那些是另外三本账。本 crate 甚至不能 `use` 它们。
//!
//! 四个文件：domain 格子 → ports 存取说明书 → actions 能做的事 → sqlite 真写入。

pub mod actions;
pub mod adapters;
pub mod domain;
pub mod ports;

pub use actions::Reserving;
pub use adapters::sqlite::SqliteReservationStore;
pub use domain::{
    CancelError, GrantError, NoShowError, Reservation, ReservationId, ReserveError, Status,
};
