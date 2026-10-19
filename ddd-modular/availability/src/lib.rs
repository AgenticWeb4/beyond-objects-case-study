//! 【小白】Availability 限界上下文：门口还挂着哪些时段牌。
//!
//! 这里**没有**「已被预约」列。占用在 Booking 的 `SlotOccupancy`。
//! 所以 Alice 订完，这里的搜索默认还列得出来——和概念侧看起来像，
//! 但原因是「预测上下文不管承诺」，不是「一台 Availability 概念机」。

pub mod domain;
pub mod ports;
pub mod service;
pub mod sqlite;

pub use domain::{CreateError, Slot, ToggleError};
pub use restaurant_ddd_kernel::{SlotId, VenueId};
pub use service::AvailabilityService;
pub use sqlite::SqliteSlotStore;
