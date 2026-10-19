//! 【小白】这是「订单」这个类。字段直接引用 UserId / RestaurantId / SlotId。
//!
//! 本 crate **依赖** `user` 和 `restaurant`——对象图穿过了包边界。
//! 概念 crate 互不依赖；这边分包之后照样互相指着。

use restaurant_oop_restaurant::{RestaurantId, SlotId};
use restaurant_oop_shared::{Date, TimeOfDay};
use restaurant_oop_user::UserId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReservationId(pub String);

impl std::fmt::Display for ReservationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReservationStatus {
    Booked,
    Canceled,
    Granted,
}

impl ReservationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ReservationStatus::Booked => "booked",
            ReservationStatus::Canceled => "canceled",
            ReservationStatus::Granted => "granted",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "booked" => Some(Self::Booked),
            "canceled" => Some(Self::Canceled),
            "granted" => Some(Self::Granted),
            _ => None,
        }
    }
}

/// 一张订单。「去订」不在它身上，而在 booking crate 的 BookingService。
#[derive(Clone, Debug)]
pub struct Reservation {
    pub id: ReservationId,
    pub user_id: UserId,
    pub restaurant_id: RestaurantId,
    pub slot_id: SlotId,
    pub time: TimeOfDay,
    pub date: Date,
    pub party_size: u32,
    pub status: ReservationStatus,
}
