//! 【小白】空位牌：门口挂着「今晚 7:30 还能坐 2–6 人」。
//!
//! 有人已经拿到预约确认，**不等于** 这张牌会被摘掉。
//! 本账本故意不看订座本。Alice 订完还能搜到，就是这个原因。

pub mod actions;
pub mod adapters;
pub mod domain;
pub mod ports;

pub use actions::Availability;
pub use adapters::sqlite::SqliteSlotStore;
pub use domain::{CreateError, Slot, SlotId, ToggleError};
