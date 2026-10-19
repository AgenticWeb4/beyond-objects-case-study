//! 【小白】Booking 限界上下文：订位这件事的对象拼图。
//!
//! 不是一台 Reserving 机器。这里仍是三个小聚合 + 一个没宿主的领域服务：
//! - [`Guest`]：订位用的客人投影（UserId + 哪天订过）
//! - [`SlotOccupancy`]：这个时段有没有被占
//! - [`Reservation`]：订单本身
//! - [`BookingService`]：同时改上面三个——「预约」仍然没有自然的家
//!
//! 概念侧把两条不变量写在同一本订座本上。打开本 crate 才能看见差别。

pub mod booking_service;
pub mod domain;
pub mod ports;
pub mod service;
pub mod sqlite;

pub use booking_service::BookingService;
pub use domain::{
    CancelError, GrantError, Guest, NoShowError, Reservation, ReservationStatus, ReserveError,
    SlotOccupancy,
};
pub use restaurant_ddd_kernel::ReservationId;
pub use service::BookingFacade;
pub use sqlite::SqliteBookingStore;
